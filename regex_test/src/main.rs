use fancy_regex::Regex as FRegex;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_SYMBOL: FRegex = FRegex::new(r"^.*symbol=\K.*?(?=&|$)").unwrap();
    static ref RE_INTERVAL: FRegex = FRegex::new(r"^.*interval=\K.*?(?=&|$)").unwrap();
}

/// fancy_regex的测试
fn fancy_match() {
    let _str1 =
        "https://data.vision/data/spot/daily/klines/1INCHUSDT/1m/1INCHUSDT-1m-2021-09-03.zip";
    let str2 = "https://api.com/api/v3/klines?symbol=BTCUSDT&interval=1m&startTime=1609430400000&endTime=1619430400000&limit=1000";

    let res1 = RE_SYMBOL.captures(str2).unwrap().unwrap();
    let res2 = RE_INTERVAL.captures(str2).unwrap().unwrap();
    println!(
        "{} {}",
        res1.get(0).unwrap().as_str(),
        res2.get(0).unwrap().as_str()
    );
}

/// regex的测试
fn re_match() {
    let _str1 =
        "https://data.vision/data/spot/daily/klines/1INCHUSDT/1m/1INCHUSDT-1m-2021-09-03.zip";
    let str2 = "https://api.com/api/v3/klines?symbol=BTCUSDT&interval=1m&startTime=1609430400000&endTime=1619430400000&limit=1000";

    let re_sym = Regex::new(r"^.*symbol=(.*?)(?:&|$)").unwrap();
    let re_intv = Regex::new(r"^.*interval=(.*?)(?:&|$)").unwrap();

    let res1 = re_sym.captures(str2).unwrap();
    let res2 = re_intv.captures(str2).unwrap();
    println!(
        "{} {}",
        res1.get(1).unwrap().as_str(),
        res2.get(1).unwrap().as_str()
    );
}

fn main() {
    re_match()
}
