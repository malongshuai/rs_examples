use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Deserialize, Serialize, EnumIter)]
pub enum IntervalType {
    Min1,
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
}

impl IntervalType {
    /// 该类型的间隔(秒)
    pub fn kline_distance(&self) -> u64 {
        match self {
            IntervalType::Min1 => 60,
            IntervalType::Min5 => 300,
            IntervalType::Min15 => 900,
            IntervalType::Min30 => 1800,
            IntervalType::Hour1 => 3600,
            IntervalType::Hour2 => 7200,
            IntervalType::Hour4 => 14400,
            IntervalType::Hour6 => 21600,
            IntervalType::Hour8 => 28800,
            IntervalType::Hour12 => 43200,
            IntervalType::Day1 => 86400,
        }
    }

    pub fn all_types() -> &'static [IntervalType] {
        &[
            IntervalType::Min1,
            IntervalType::Min5,
            IntervalType::Min15,
            IntervalType::Min30,
            IntervalType::Hour1,
            IntervalType::Hour2,
            IntervalType::Hour4,
            IntervalType::Hour6,
            IntervalType::Hour8,
            IntervalType::Hour12,
            IntervalType::Day1,
        ]
    }
}

impl From<&str> for IntervalType {
    fn from(str: &str) -> Self {
        match str {
            "1m" | "1min" | "min1" => Self::Min1,
            "5m" | "5min" | "min5" => Self::Min5,
            "15m" | "15min" | "min15" => Self::Min15,
            "30m" | "30min" | "min30" => Self::Min30,
            "60m" | "60min" | "min60" => Self::Hour1,
            "1h" | "1hour" | "hour1" => Self::Hour1,
            "2h" | "2hour" | "hour2" => Self::Hour2,
            "4h" | "4hour" | "hour4" => Self::Hour4,
            "6h" | "6hour" | "hour6" => Self::Hour6,
            "8h" | "8hour" | "hour8" => Self::Hour8,
            "12h" | "12hour" | "hour12" => Self::Hour12,
            "1d" | "1day" | "day1" => Self::Day1,
            _ => panic!("`{}` Can't Convert to IntervalType", str),
        }
    }
}

impl Display for IntervalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Min1 => "min1",
            Self::Min5 => "min5",
            Self::Min15 => "min15",
            Self::Min30 => "min30",
            Self::Hour1 => "hour1",
            Self::Hour2 => "hour2",
            Self::Hour4 => "hour4",
            Self::Hour6 => "hour6",
            Self::Hour8 => "hour8",
            Self::Hour12 => "hour12",
            Self::Day1 => "day1",
        };

        write!(f, "{}", s)
    }
}
