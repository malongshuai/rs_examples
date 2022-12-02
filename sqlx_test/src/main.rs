//! 连接池(Pool)可跨线程使用，单个连接(Connection)无法跨线程使用  
//! 连接池包括：Pool, SqlitePool, PgPool, MySQLPool  
//! 单个连接包括: Connection, SqliteConnection, ...  
//! Pg使用$1 $2 $3...$N 作为sql语句的占位符，其它数据库使用?作为占位符，提供占位符参数的方式是使用bind()方法  
//! 使用查询宏(例如`query!()`)，可像format!()一样绑定参数，且在编译器将语句和参数进行编译，需要设置环境变量DATABASE_URL  
//! query()类的查询自动prepare  
//! 查询返回的Row，比较难直接处理，可在类型上实现`Trait FromRow`来简化Row转换为数据结构，参考kl.rs中的实现  
#![allow(dead_code)]

pub mod kl;

use std::str::FromStr;

use kl::KLine;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    ConnectOptions, FromRow, MySqlPool, Row, SqlitePool,
};
use tracing::info;

#[derive(Debug, Clone)]
struct SqliteDb {
    pool: SqlitePool,
}

impl SqliteDb {
    async fn new(path: &str) -> Self {
        let path = format!("sqlite://{}", path);
        let mut opts = SqliteConnectOptions::from_str(&path)
            .unwrap()
            .journal_mode(SqliteJournalMode::Off)
            .synchronous(SqliteSynchronous::Off)
            .create_if_missing(true);
        opts.log_statements(log::LevelFilter::Off)
            .log_slow_statements(log::LevelFilter::Warn, std::time::Duration::from_millis(5));
        let pool = SqlitePoolOptions::new().connect_with(opts).await.unwrap();

        Self { pool }
    }

    fn create_table_sql(&self, table_name: &str) -> String {
        format!(
            "create table if not exists {}(
                            epoch       int     not null primary key,
                            close_epoch int     not null,
                            symbol      text    not null,
                            interval    text    not null,
                            finish      int     not null,
                            open        real not null,
                            high        real not null,
                            close       real not null,
                            low         real not null,
                            amount      real not null,
                            vol         real not null,
                            count       int not null
                        );",
            table_name
        )
    }

    async fn create_table(&mut self, table_name: &str) {
        let sql = self.create_table_sql(table_name);
        sqlx::query(&sql).execute(&self.pool).await.unwrap();
    }

    async fn update(&mut self, kl: KLine, table_name: &str) {
        let stmt_str = format!("{}
            insert or
            replace into {}
            (epoch, close_epoch, symbol, interval, finish, high, open, low, close, count, amount, vol)
            values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        ", self.create_table_sql(table_name), table_name);

        let intv = kl.interval.as_str();
        let finish = kl.finish as i32;
        sqlx::query(&stmt_str)
            .bind(kl.epoch as i64)
            .bind(kl.close_epoch as i64)
            .bind(kl.symbol)
            .bind(intv)
            .bind(finish)
            .bind(kl.high)
            .bind(kl.open)
            .bind(kl.low)
            .bind(kl.close)
            .bind(kl.count as i64)
            .bind(kl.amount)
            .bind(kl.vol)
            .execute(&self.pool)
            .await
            .unwrap_or_else(|e| panic!("error: `{}`, {}", stmt_str, e));
    }

    async fn query(&self, table_name: &str) -> Vec<KLine> {
        let sql = format!("select * from {} where epoch > ?;", table_name);
        let res = sqlx::query(&sql)
            .bind(0_i32)
            .fetch_all(&self.pool)
            .await
            .unwrap();

        let mut ks = vec![];
        for row in res {
            ks.push(KLine::from_row(&row).unwrap());
        }
        ks
    }

    async fn first_epoch(&self, table_name: &str) -> u64 {
        let sql = format!("select epoch from {} order by epoch limit 1;", table_name);
        let res = sqlx::query(&sql).fetch_one(&self.pool).await.unwrap();

        res.get::<i64, _>(0) as u64
    }
}

#[derive(Debug, Clone)]
struct MySqlDb {
    pool: MySqlPool,
}

impl MySqlDb {
    async fn new() -> Self {
        let conn = MySqlPool::connect("mysql://root:mls@gxf1129@127.0.0.1")
            .await
            .unwrap();
        Self { pool: conn }
    }

    async fn databases(&self) -> Vec<String> {
        let inner_dbs = ["information_schema", "mysql", "sys", "performance_schema"];
        let res = sqlx::query("show databases;")
            .fetch_all(&self.pool)
            .await
            .unwrap();
        let mut ks = vec![];
        for row in res {
            let db = row.get::<String, _>(0);
            if !inner_dbs.contains(&db.as_str()) {
                ks.push(db);
            }
        }
        ks
    }

    async fn create_database(&self, db_name: &str) {
        let sql_str = format!("create database if not exists {};", db_name);
        let res = sqlx::query(&sql_str).execute(&self.pool).await;
        info!("{:?}", res);
    }

    async fn create_table(&self, db_name: &str, tbl_name: &str) {
        let sql_str = format!(
            "create table if not exists {}.{}(
                            `epoch`       bigint not null primary key,
                            `close_epoch` bigint not null,
                            `symbol`      char(20) not null,
                            `interval`    char(5) not null,
                            `finish`      tinyint UNSIGNED  not null,
                            `open`        real not null,
                            `high`        real not null,
                            `close`       real not null,
                            `low`         real not null,
                            `amount`      real not null,
                            `vol`         real not null,
                            `count`       bigint not null
                        ) engine=myisam;",
            db_name, tbl_name
        );

        let res = sqlx::query(&sql_str).execute(&self.pool).await;
        info!("{:?}", res);
    }

    async fn update(&mut self, db_name: &str, tbl_name: &str, kl: KLine) {
        let stmt_str = format!("
            replace into {}.{}
            (epoch, close_epoch, symbol, `interval`, finish, high, `open`, low, `close`, `count`, amount, vol)
            values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        ", db_name, tbl_name);

        let intv = kl.interval.as_str();
        // let finish = kl.finish as i32;
        sqlx::query(&stmt_str)
            .bind(kl.epoch as i64)
            .bind(kl.close_epoch as i64)
            .bind(kl.symbol)
            .bind(intv)
            .bind(kl.finish)
            .bind(kl.high)
            .bind(kl.open)
            .bind(kl.low)
            .bind(kl.close)
            .bind(kl.count as i64)
            .bind(kl.amount)
            .bind(kl.vol)
            .execute(&self.pool)
            .await
            .unwrap_or_else(|e| panic!("error: `{}`, {}", stmt_str, e));
    }

    async fn query(&self, db_name: &str, tbl_name: &str) -> Vec<KLine> {
        let sql = format!("select * from {}.{} where epoch > ?;", db_name, tbl_name);
        let res = sqlx::query(&sql)
            .bind(0_i32)
            .fetch_all(&self.pool)
            .await
            .unwrap();

        let mut ks = vec![];
        for row in res {
            ks.push(KLine::from_row(&row).unwrap());
        }
        ks
    }

    async fn first_epoch(&self, table_name: &str) -> u64 {
        let sql = format!("select epoch from {} order by epoch limit 1;", table_name);
        let res = sqlx::query(&sql).fetch_one(&self.pool).await.unwrap();

        res.get::<i64, _>(0) as u64
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .init();

    let mysql_db = MySqlDb::new().await;
    let tbl_names = vec!["min1", "min5"];
    mysql_db.create_database("newdb").await;
    for tbl_name in tbl_names {
        mysql_db.create_table("newdb", tbl_name).await;
    }

    dbg!(mysql_db.databases().await);

    // let mut db = mysql_db.clone();
    // let t = tokio::spawn(async move {
    //     let (kl1, kl2) = two_kline();
    //     db.update("newdb", "min1", kl1).await;
    //     db.update("newdb", "min5", kl2).await;
    // });
    // t.await.unwrap();

    // let ks = mysql_db.query("newdb", "min1").await;
    // println!("{:?}", ks);

    // let mut sql_db = SqliteDb::new("test.db").await;
    // let table_name = "min1";
    // sql_db.create_table(table_name).await;

    // let mut db = sql_db.clone();
    // let t = tokio::spawn(async move {
    //     let (kl1, kl2) = two_kline();
    //     db.update(kl1, table_name).await;
    //     db.update(kl2, table_name).await;
    // });

    // t.await.unwrap();
    // let ks = sql_db.query(table_name).await;
    // let start_epoch = sql_db.first_epoch(table_name).await;
    // println!("{}", start_epoch);
    // println!("{:?}", ks);
}

fn two_kline() -> (KLine, KLine) {
    let kl1 = KLine {
        symbol: "ZILUSDT".to_string(),
        interval: kl::KLineInterval::Hour1,
        epoch: 1640793600000,
        close_epoch: 1640797199999,
        finish: true,
        open: 0.07996,
        high: 0.08013,
        low: 0.0787,
        close: 0.07991,
        amount: 997416.976317,
        vol: 12561514.4,
        count: 3650,
    };

    let kl2 = KLine {
        symbol: "ZILUSDT".to_string(),
        interval: kl::KLineInterval::Hour1,
        epoch: 1640797200000,
        close_epoch: 1640800799999,
        finish: true,
        open: 0.07989,
        high: 0.0805,
        low: 0.07911,
        close: 0.07976,
        amount: 0.0,
        vol: 14049941.4,
        count: 4102,
    };
    (kl1, kl2)
}
