use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::{interval::IntervalType, kline::Kline};

pub(crate) struct KlineRow {
    pub id: u64,
    pub close: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub count: f64,
    pub amount: f64,
    pub vol: f64,
}

impl KlineRow {
    pub(crate) fn kline(self, kl_type: IntervalType) -> Kline {
        Kline {
            id: self.id,
            interval: kl_type,
            close: self.close,
            open: self.open,
            high: self.high,
            low: self.low,
            count: self.count,
            amount: self.amount,
            vol: self.vol,
        }
    }
}

impl<'r> FromRow<'r, SqliteRow> for KlineRow {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: i64 = row.get_unchecked(0);
        let high: f64 = row.get_unchecked(1);
        let open: f64 = row.get_unchecked(2);
        let low: f64 = row.get_unchecked(3);
        let close: f64 = row.get_unchecked(4);
        let count: i64 = row.get_unchecked(5);
        let amount: f64 = row.get_unchecked(6);
        let vol: f64 = row.get_unchecked(7);

        let kl = Self {
            id: id as u64,
            close,
            open,
            high,
            low: low as f64,
            count: count as f64,
            amount,
            vol,
        };
        Ok(kl)
    }
}
