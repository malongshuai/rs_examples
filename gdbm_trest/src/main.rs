#![allow(dead_code)]
#![allow(unused_imports)]

use gdbm::{Gdbm, Open};
use gdbm_my::GdbmOpener as GdbmOpenerMy;
use gnudbm::GdbmOpener;
use nix::{libc, sys::wait::waitpid, unistd::Pid};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{env::args, path::PathBuf, time::Instant};

#[derive(Debug, Deserialize, Serialize)]
struct Keys(Vec<u32>);

#[derive(Debug, Deserialize, Serialize)]
struct Kline {
    id: u64,
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    count: f64,
    amount: f64,
    vol: f64,
}

#[tokio::main]
async fn main() {
    // let sym = "ankrusdt_1min";
    // let key = "1609430400";
    // read_gdbm::<Vec<Kline>>(sym, key);
    // read_gnudbm::<Vec<Kline>>(sym, key);
    // read_gdbm_my::<Vec<Kline>>(sym, key);
    // let key = "max_epoch";
    // read_gdbm::<u64>(sym, key);
    // read_gnudbm::<u64>(sym, key);
    // read_gdbm_my::<u64>(sym, key);

    let syms = vec![
        "ankrusdt_1min",
        "btcusdt_1min",
        "aaveusdt_1min",
        "ankrusdt_1min",
        "btcusdt_1min",
        "aaveusdt_1min",
    ];
    chuan_test(syms.clone());
    multi_process(syms.clone());
    multi_thread(syms.clone());
    async_test(syms.clone()).await;
    // let x = args().skip(1).take(1).collect::<String>();
    // let f = format!("{}usdt_1min", x);
    // read_gdbm_my::<Vec<Kline>>(&f, "1609430400");
}

async fn async_test(syms: Vec<&str>) {
    println!("--------异步多线程----------");
    let mut ts = vec![];
    for sym in syms {
        let sym = sym.to_string();
        let t = tokio::spawn(async move {
            read_gdbm_my::<Vec<Kline>>(&sym, "1609430400");
        });
        ts.push(t);
    }
    for t in ts {
        t.await.unwrap();
    }
}

fn chuan_test(syms: Vec<&str>) {
    println!("-------串行-----------");
    for sym in syms {
        read_gdbm_my::<Vec<Kline>>(sym, "1609430400");
    }
}

fn multi_thread(syms: Vec<&str>) {
    println!("--------多线程----------");
    let mut ts = vec![];
    for sym in syms {
        let sym = sym.to_string();
        let t = std::thread::spawn(move || {
            read_gdbm_my::<Vec<Kline>>(&sym, "1609430400");
        });
        ts.push(t);
    }
    for t in ts {
        t.join().unwrap();
    }
}

fn multi_process(syms: Vec<&str>) {
    println!("--------多进程----------");
    let mut children = vec![];
    for f in syms {
        let pid = match unsafe { nix::unistd::fork().unwrap() } {
            nix::unistd::ForkResult::Parent { child } => child,
            nix::unistd::ForkResult::Child => {
                read_gdbm_my::<Vec<Kline>>(f, "1609430400");
                unsafe { libc::_exit(0) };
            }
        };
        children.push(pid);
    }
    for pid in children {
        waitpid(pid, None).unwrap();
    }
}

fn read_gdbm<T>(sym: &str, key: &str)
where
    T: DeserializeOwned,
{
    let path = PathBuf::from(sym);
    let db = Gdbm::new(&path, 0, Open::READER, 0o666).unwrap();
    let start = Instant::now();
    let content = db.fetch(key).unwrap();
    let read_time = start.elapsed().as_micros();
    let _res = serde_json::from_str::<T>(&content).unwrap();
    let de_time = start.elapsed().as_micros() - read_time;
    println!(
        "GDBM: key: {}, read_time: {}, de_time: {}, ",
        key, read_time, de_time,
    );
}

fn read_gnudbm<T>(sym: &str, key: &str)
where
    T: DeserializeOwned,
{
    let path = PathBuf::from(sym);
    let db = GdbmOpener::new()
        .create(true)
        .readonly(&path)
        .expect("db creation failed");
    let start = Instant::now();
    let entry = db.fetch(key).unwrap();
    let read_time = start.elapsed().as_micros();
    let _res = serde_json::from_slice::<T>(entry.as_bytes()).unwrap();
    let de_time = start.elapsed().as_micros() - read_time;
    println!(
        "GnuDBM: key: {}, read_time: {}, de_time: {}",
        key, read_time, de_time
    );
}

fn read_gdbm_my<T>(sym: &str, key: &str)
where
    T: DeserializeOwned,
{
    let path = PathBuf::from(sym);
    let db = GdbmOpenerMy::new()
        .create(true)
        .pre_read(true)
        .readonly(&path)
        .expect("db creation failed");

    let start = Instant::now();
    let entry = db.fetch(key).unwrap();
    let read_time = start.elapsed().as_micros();
    let str = String::from_utf8(entry.as_bytes().to_vec()).unwrap();
    let _res = serde_json::from_str::<T>(&str).unwrap();
    let de_time = start.elapsed().as_micros() - read_time;
    println!(
        "GDBM_My: key: {}, read_time: {}, de_time: {}",
        key, read_time, de_time
    );
}
