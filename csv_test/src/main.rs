use std::{
    fmt,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader},
};

use serde::{Deserialize, Serialize};

fn main() {
    let path = "/mnt/g/桌面/a.csv";
    let kl1 = KLine {
        symbol: "BTCUSDT".to_string(),
        interval: KLineInterval::Min1,
        epoch: 1639989540000,
        close_epoch: 1639989599999,
        finish: true,
        open: 1.2,
        high: 1.3,
        low: 1.19,
        close: 1.25,
        amount: 8999.192,
        vol: 10004.2,
        count: 2003,
    };

    // let wtr = OpenOptions::new()
    //     .create(true)
    //     .append(true)
    //     .open(path)
    //     .unwrap();
    // // let mut csv_wr = csv::Writer::from_writer(wtr);
    // let mut csv_wr = csv::WriterBuilder::new().has_headers(true).from_writer(wtr);
    // csv_wr.serialize(kl1).unwrap();
    // csv_wr.flush().unwrap();

    // let mut csv_rd = csv::ReaderBuilder::new()
    //     .has_headers(true)
    //     .from_path(path)
    //     .unwrap();
    // for res in csv_rd.deserialize::<KLine>() {
    //     println!("{:?}", res);
    // }

    let mut reader = BufReader::new(File::open(path).unwrap());
    let mut ks = Vec::new();
    let mut buf = String::new();
    loop {
        let n = reader.read_line(&mut buf).unwrap();
        if n == 0 {
            break;
        }

        let str = buf.trim_end(); // 去掉尾部换行符
        println!("{:?}", str);
        let new_str = format!("[{}]", str);
        let x = serde_json::from_str::<KLine>(&new_str).unwrap();
        println!("{:?}", x);
        let s: Vec<&str> = str.split(',').collect();
        ks.push(KLine {
            symbol: s[0].to_string(),
            interval: KLineInterval::from(s[1]),
            epoch: s[2].parse().unwrap(),
            close_epoch: s[3].parse().unwrap(),
            finish: s[4].parse().unwrap(),
            open: s[5].parse().unwrap(),
            high: s[6].parse().unwrap(),
            low: s[7].parse().unwrap(),
            close: s[8].parse().unwrap(),
            amount: s[9].parse().unwrap(),
            vol: s[10].parse().unwrap(),
            count: s[11].parse().unwrap(),
        });
        buf.clear();
    }
    println!("{:?}", ks);
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(from = "WrapKLine")]
pub struct KLine {
    /// 交易对
    pub symbol: String,
    /// K线间隔
    pub interval: KLineInterval,
    /// 开盘时间
    pub epoch: u64,
    /// 收盘时间
    pub close_epoch: u64,
    /// 该K线是否已经收盘
    pub finish: bool,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量
    pub amount: f64,
    /// 成交额
    pub vol: f64,
    /// 成交笔数
    pub count: u64,
}

impl From<WrapKLine> for KLine {
    fn from(wk: WrapKLine) -> Self {
        match wk {
            WrapKLine::RawKLine(data) => Self {
                symbol: data.symbol,
                epoch: data.epoch,
                close_epoch: data.close_epoch,
                high: data.high,
                close: data.close,
                low: data.low,
                open: data.open,
                count: data.count,
                amount: data.amount,
                vol: data.vol,
                finish: data.finish,
                interval: data.interval,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum WrapKLine {
    RawKLine(RawKLine),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawKLine {
    /// 交易对
    pub symbol: String,
    /// K线间隔
    pub interval: KLineInterval,
    /// 开盘时间
    pub epoch: u64,
    /// 收盘时间
    pub close_epoch: u64,
    /// 该K线是否已经收盘
    pub finish: bool,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量
    pub amount: f64,
    /// 成交额
    pub vol: f64,
    /// 成交笔数
    pub count: u64,
}

/// K线间隔，包括：1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KLineInterval {
    /// 1分钟间隔
    #[serde(rename = "1m")]
    Min1 = 60_000,
    /// 3分钟间隔
    #[serde(rename = "3m")]
    Min3 = 180_000,
    /// 5分钟间隔
    #[serde(rename = "5m")]
    Min5 = 300_000,
    /// 15分钟间隔
    #[serde(rename = "15m")]
    Min15 = 900_000,
    /// 30分钟间隔
    #[serde(rename = "30m")]
    Min30 = 1_800_000,
    /// 1小时间隔
    #[serde(rename = "1h")]
    Hour1 = 3_600_000,
    /// 2小时间隔
    #[serde(rename = "2h")]
    Hour2 = 7_200_000,
    /// 4小时间隔
    #[serde(rename = "4h")]
    Hour4 = 14_400_000,
    /// 6小时间隔
    #[serde(rename = "6h")]
    Hour6 = 21_600_000,
    /// 8小时间隔
    #[serde(rename = "8h")]
    Hour8 = 28_800_000,
    /// 12小时间隔
    #[serde(rename = "12h")]
    Hour12 = 43_200_000,
    /// 1天间隔
    #[serde(rename = "1d")]
    Day1 = 86_400_000,
    /// 3天间隔
    #[serde(rename = "3d")]
    Day3 = 3 * 86_400_000,
    /// 1周间隔
    #[serde(rename = "1w")]
    Week1 = 7 * 86_400_000,
    // /// 1月间隔
    // #[serde(rename = "1M")]
    // Mon1,
}

impl KLineInterval {
    pub fn is_intv(interval: &str) -> bool {
        let valid_interval = [
            "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d", "3d", "1w",
            "1M",
        ];
        valid_interval.contains(&interval)
    }
}

impl fmt::Display for KLineInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Min1 => write!(f, "1m"),
            Self::Min3 => write!(f, "3m"),
            Self::Min5 => write!(f, "5m"),
            Self::Min15 => write!(f, "15m"),
            Self::Min30 => write!(f, "30m"),
            Self::Hour1 => write!(f, "1h"),
            Self::Hour2 => write!(f, "2h"),
            Self::Hour4 => write!(f, "4h"),
            Self::Hour6 => write!(f, "6h"),
            Self::Hour8 => write!(f, "8h"),
            Self::Hour12 => write!(f, "12h"),
            Self::Day1 => write!(f, "1d"),
            Self::Day3 => write!(f, "3d"),
            Self::Week1 => write!(f, "1w"),
        }
    }
}

impl From<&str> for KLineInterval {
    fn from(s: &str) -> Self {
        match s {
            "1m" => Self::Min1,
            "3m" => Self::Min3,
            "5m" => Self::Min5,
            "15m" => Self::Min15,
            "30m" => Self::Min30,
            "1h" => Self::Hour1,
            "2h" => Self::Hour2,
            "4h" => Self::Hour4,
            "6h" => Self::Hour6,
            "8h" => Self::Hour8,
            "12h" => Self::Hour12,
            "1d" => Self::Day1,
            "3d" => Self::Day3,
            "1w" => Self::Week1,
            _ => panic!("unsupported kline interval"),
        }
    }
}
