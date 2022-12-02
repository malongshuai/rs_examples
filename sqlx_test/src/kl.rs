
use sqlx::{FromRow, Row, mysql::MySqlRow, sqlite::SqliteRow};

#[derive(Debug, Clone)]
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

impl<'r> FromRow<'r, SqliteRow> for KLine {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let epoch: i64 = row.try_get(0)?;
        let close_epoch: i64 = row.get(1);
        let symbol: String = row.get(2);
        let interval: &str = row.get(3);
        let finish: u8 = row.get(4);
        let open: f64 = row.get(5);
        let high: f64 = row.get(6);
        let close: f64 = row.get(7);
        let low: f64 = row.get(8);
        let amount: f64 = row.get(9);
        let vol: f64 = row.get(10);
        let count: i64 = row.get(11);

        Ok(KLine {
            symbol,
            interval: KLineInterval::from(interval),
            epoch: epoch as u64,
            close_epoch: close_epoch as u64,
            finish: finish != 0,
            open,
            high,
            low,
            close,
            amount,
            vol,
            count: count as u64,
        })
    }
}

impl<'r> FromRow<'r, MySqlRow> for KLine {
    fn from_row(row: &'r MySqlRow) -> Result<Self, sqlx::Error> {
        let epoch: i64 = row.try_get(0)?;
        let close_epoch: i64 = row.get(1);
        let symbol: String = row.get(2);
        let interval: &str = row.get(3);
        let finish: u8 = row.get(4);
        let open: f64 = row.get(5);
        let high: f64 = row.get(6);
        let close: f64 = row.get(7);
        let low: f64 = row.get(8);
        let amount: f64 = row.get(9);
        let vol: f64 = row.get(10);
        let count: i64 = row.get(11);

        Ok(KLine {
            symbol,
            interval: KLineInterval::from(interval),
            epoch: epoch as u64,
            close_epoch: close_epoch as u64,
            finish: finish != 0,
            open,
            high,
            low,
            close,
            amount,
            vol,
            count: count as u64,
        })
    }
}

/// K线间隔，包括：1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KLineInterval {
    Min1,
    Min3,
    Min5,
    Min15,
    Min30,
    Hour1,
    Hour2,
    Hour4,
    Hour6,
    Hour8,
    Hour12,
    Day1,
    Day3,
    Week1,
}

impl KLineInterval {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Min1 => "1m",
            Self::Min3 => "3m",
            Self::Min5 => "5m",
            Self::Min15 => "15m",
            Self::Min30 => "30m",
            Self::Hour1 => "1h",
            Self::Hour2 => "2h",
            Self::Hour4 => "4h",
            Self::Hour6 => "6h",
            Self::Hour8 => "8h",
            Self::Hour12 => "12h",
            Self::Day1 => "1d",
            Self::Day3 => "3d",
            Self::Week1 => "1w",
        }
    }
}

impl std::fmt::Display for KLineInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
