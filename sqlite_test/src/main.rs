use rusqlite::{params, Connection};

#[derive(Debug)]
struct Kline {
    epoch: u64,
    close_epoch: u64,
    high: f64,
    open: f64,
    low: f64,
    close: f64,
    amount: f64,
    vol: f64,
}

fn test() {
    let conn = Connection::open("a.db").unwrap();
    conn.execute(
        "
          create table if not exists btcusdt_1min (
            epoch int primary key not null,
            close_epoch int not null,
            high numeric not null,
            open numeric not null,
            low numeric not null,
            close numeric not null,
            amount numeric not null,
            vol numeric not null
          );
        ",
        [],
    )
    .unwrap();

    let kl = Kline {
        epoch: 34567,
        close_epoch: 78961,
        high: 35333.3,
        open: 34443.2,
        low: 33333.3,
        close: 34333.3,
        amount: 23.0,
        vol: 682023.3,
    };

    let mut insert_stmt = conn
        .prepare_cached(
            "
              insert into btcusdt_1min(epoch, close_epoch, high, open, low, close, amount, vol) 
              values (?, ?, ?, ?, ?, ?, ?, ?);
            ",
        )
        .unwrap();
    insert_stmt
        .execute(params![
            kl.epoch,
            kl.close_epoch,
            kl.high,
            kl.open,
            kl.low,
            kl.close,
            kl.amount,
            kl.vol
        ])
        .unwrap();

    let mut select_stmt = conn
        .prepare_cached(
            "
              select * from btcusdt_1min;
            ",
        )
        .unwrap();

    let res = select_stmt
        .query([])
        .unwrap()
        .mapped(|r| {
            Ok(Kline {
                epoch: r.get(0).unwrap(),
                close_epoch: r.get(1).unwrap(),
                high: r.get(2).unwrap(),
                open: r.get(3).unwrap(),
                low: r.get(4).unwrap(),
                close: r.get(5).unwrap(),
                amount: r.get(6).unwrap(),
                vol: r.get(7).unwrap(),
            })
        })
        .collect::<Vec<Result<Kline, rusqlite::Error>>>();
    dbg!(res);
}

fn main() {
    test();
}
