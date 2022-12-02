pub mod interval;
pub mod kline;
pub mod kline_row;

use std::{path::Path, str::FromStr};

use interval::IntervalType;
use kline::Klines;
use kline_row::KlineRow;
use sqlx::{
    sqlite::{
        SqliteJournalMode, SqliteLockingMode, SqlitePoolOptions, SqliteQueryResult, SqliteRow,
        SqliteSynchronous,
    },
    ConnectOptions, Database, Error, FromRow, Row, Sqlite, SqlitePool,
};
use strum::IntoEnumIterator;

const SQL_DB_DIR: &str = "/mnt/td3/bian_ks_shm";

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct SQL_DB {
    conn: SqlitePool,
    sym: String,
}

impl SQL_DB {
    /// 传递不区分大小写的Symbol Name
    pub async fn new(sym: &str) -> Self {
        Self::open(sym).await
    }

    /// 传递不区分大小写的Symbol Name
    pub async fn open(sym: &str) -> Self {
        let db_name = format!("{}.db", sym.to_lowercase());
        let db_path = Path::new(SQL_DB_DIR).join(&db_name);

        let mut conn_opt = sqlx::sqlite::SqliteConnectOptions::from_str(db_path.to_str().unwrap())
            .unwrap()
            .journal_mode(SqliteJournalMode::Off)
            .synchronous(SqliteSynchronous::Off)
            .locking_mode(SqliteLockingMode::Exclusive)
            .page_size(4096)
            .create_if_missing(true);
        conn_opt.disable_statement_logging();

        // 对于sqlite来说，不要设置过多连接，会导致打开文件数量过大
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(conn_opt)
            .await
            .unwrap_or_else(|e| panic!("open db failed: {}", e));

        let s = Self {
            conn: pool,
            sym: sym.into(),
        };

        s
    }

    /// 根据K线类型，返回对应的表名
    fn table_name(&self, kl_type: IntervalType) -> String {
        kl_type.to_string()
    }

    fn create_table_sql(&self, kl_type: IntervalType) -> String {
        let table_name = self.table_name(kl_type);
        format!(
            "
                  CREATE TABLE IF NOT EXISTS {}
                  (
                      ID      INT     NOT NULL PRIMARY KEY,
                      HIGH    NUMERIC NOT NULL,
                      OPEN    NUMERIC NOT NULL,
                      LOW     NUMERIC NOT NULL,
                      CLOSE   NUMERIC NOT NULL,
                      COUNT   NUMERIC NOT NULL,
                      AMOUNT  NUMERIC NOT NULL,
                      VOL     NUMERIC NOT NULL
                  );",
            table_name
        )
    }

    /// 建表，表存在则不管
    async fn create_table_if_not_exist(&self, kl_type: IntervalType) -> &Self {
        let table_name = self.table_name(kl_type);
        let sql = self.create_table_sql(kl_type);
        sqlx::query(&sql)
            .execute(&self.conn)
            .await
            .unwrap_or_else(|e| panic!("create table({}.{}) failed: {}", self.sym, table_name, e));
        self
    }

    /// 表是否存在
    #[allow(dead_code)]
    async fn table_exists(&self, kl_type: IntervalType) -> bool {
        let table_name = self.table_name(kl_type);
        let sql = "select count(*) from sqlite_master where type = ? and name = ?;";
        let res: i64 = sqlx::query(&sql)
            .bind("table")
            .bind(&table_name)
            .fetch_one(&self.conn)
            .await
            .unwrap()
            .get(0);

        res != 0
    }

    /// 最小的epoch，如果缓存中不存在，从数据库中查询并保存，如果数据库中不存在任何行，则返回0
    async fn min_epoch(&self, kl_type: IntervalType) -> Option<u64> {
        let table_name = self.table_name(kl_type);
        let sql = format!("select id from {} order by id limit ?;", table_name);
        let res = sqlx::query(&sql).bind(1).fetch_optional(&self.conn).await;
        match res {
            Ok(r) => match r {
                Some(x) => {
                    let epoch = x.get::<i64, _>(0) as u64;
                    Some(epoch)
                }
                None => None,
            },
            Err(e) => panic!("error: `{}`, {}", sql, e),
        }
    }

    async fn max_epoch(&self, kl_type: IntervalType) -> Option<u64> {
        let table_name = self.table_name(kl_type);
        let sql = format!("select id from {} order by id desc limit ?", table_name);
        let res = sqlx::query(&sql)
            .bind(1)
            .fetch_optional(&self.conn)
            .await
            .unwrap_or_else(|e| panic!("error: `{}`, {}", sql, e));

        match res {
            None => None,
            Some(x) => {
                let epoch = x.get::<i64, _>(0) as u64;
                Some(epoch)
            }
        }
    }

    pub async fn epoch_range_from_db(&self, kl_type: IntervalType) -> (u64, u64) {
        let table_name = self.table_name(kl_type);
        let start_sql = format!("select id from {} order by id limit ?", table_name);
        let end_sql = format!("select id from {} order by id desc limit ?", table_name);

        let start_epoch = sqlx::query(&start_sql)
            .bind(1)
            .fetch_one(&self.conn)
            .await
            .unwrap_or_else(|e| panic!("error: `{}`, {}", start_sql, e))
            .get::<i64, _>(0) as u64;
        let end_epoch = sqlx::query(&end_sql)
            .bind(1)
            .fetch_one(&self.conn)
            .await
            .unwrap_or_else(|e| panic!("error: `{}`, {}", start_sql, e))
            .get::<i64, _>(0) as u64;

        (start_epoch, end_epoch)
    }

    /// 调整from和to，使它们都在 epoch_range 之间
    /// 返回 (from, to, min_epoch, max_epoch)
    #[allow(clippy::wrong_self_convention)]
    async fn from_to(
        &self,
        kl_type: IntervalType,
        from: u64,
        to: u64,
    ) -> Option<(u64, u64, u64, u64)> {
        let (min_epoch, max_epoch) = self.epoch_range_from_db(kl_type).await;

        if from > max_epoch || to < min_epoch || from > to {
            return None;
        }

        Some((from.max(min_epoch), to.min(max_epoch), min_epoch, max_epoch))
    }

    async fn klines_between_from_db(&self, kl_type: IntervalType, from: u64, to: u64) -> Klines {
        let (from, to) = match self.from_to(kl_type, from, to).await {
            Some((f, t, _, _)) => (f as i64, t as i64),
            None => return Klines::new(),
        };

        let table_name = self.table_name(kl_type);
        let sql = format!(
            "select * from {} where id between ? and ? order by id;",
            table_name
        );

        let rows: sqlx::Result<Vec<SqliteRow>> = sqlx::query(&sql)
            .bind(from)
            .bind(to)
            .fetch_all(&self.conn)
            .await;
        let mut ks = Klines::new();
        match rows {
            Ok(rows) => {
                for row in rows {
                    ks.push(KlineRow::from_row(&row).unwrap().kline(kl_type));
                }
            }
            Err(e) => panic!("error: `{}`, {}, {}", sql, self.sym, e),
        }
        ks
    }
}

#[tokio::main]
async fn main() {
    let db = SQL_DB::open("aaveusdt").await;
    println!(
        "table exists: {:?}",
        db.table_exists(IntervalType::Hour1).await
    );

    println!("min_epoch: {:?}", db.min_epoch(IntervalType::Hour1).await);
    println!("max_epoch: {:?}", db.max_epoch(IntervalType::Hour1).await);
    println!(
        "max_epoch: {:?}",
        db.epoch_range_from_db(IntervalType::Hour1).await
    );
    println!(
        "all_ks_from_db: {:?}",
        db.klines_between_from_db(IntervalType::Hour1, 1609430400, 1663801200)
            .await
            .len()
    );
}
