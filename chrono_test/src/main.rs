use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};

pub fn east8() -> FixedOffset {
    FixedOffset::east(8 * 3600)
}

pub fn now8() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&east8())
}

fn main() {
    println!("{} {}", today00_0().timestamp(), today00_1().timestamp());
}
pub fn today00_0() -> DateTime<FixedOffset> {
    now8().date().and_hms(0, 0, 0)
}
pub fn today00_1() -> NaiveDateTime {
    now8().date_naive().and_hms_opt(0, 0, 0).unwrap()
}
