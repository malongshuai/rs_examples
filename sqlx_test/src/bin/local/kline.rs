use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    ops,
    slice::{Iter, Windows},
};

use crate::interval::IntervalType;

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct Kline {
    pub id: u64,
    pub interval: IntervalType,
    pub close: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub count: f64,
    pub amount: f64,
    pub vol: f64,
}

impl Kline {
    pub fn build(epoch: u64, intv_type: IntervalType) -> Self {
        Self {
            id: epoch,
            interval: intv_type,
            close: 0f64,
            open: 0f64,
            high: 0f64,
            low: 0f64,
            count: 0f64,
            amount: 0f64,
            vol: 0f64,
        }
    }

    /// 将多根K线合并成单根K线，合成的k线的id取自第一根K线。
    /// 例如，可用于将多根min1 k线合并成min15的k线
    pub fn merge_kline<T: Iterator<Item = Kline>>(
        mut ks: T,
        dest_intv: IntervalType,
    ) -> Option<Self> {
        let mut kl = ks.next()?;
        kl.interval = dest_intv;
        for k in ks {
            kl.high = kl.high.max(k.high);
            kl.low = kl.low.min(k.low);
            kl.close = k.close;
            kl.count += k.count;
            kl.amount += k.amount;
            kl.vol += k.vol;
        }

        Some(kl)
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Klines(Vec<Kline>);

impl Klines {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn windows(&self, n: usize) -> Windows<'_, Kline> {
        self.0.windows(n)
    }

    pub fn inner_data(&self) -> &Vec<Kline> {
        &self.0
    }

    pub fn as_slice(&self) -> &[Kline] {
        self.0.as_slice()
    }

    /// 必须传递已经对齐了k线Epoch的值作为参数
    pub fn index(&self, id: u64) -> Option<usize> {
        self.0.binary_search_by(|x| x.id.cmp(&id)).ok()
    }

    /// 返回某个具体Epoch时间点的K线
    pub fn kline_at(&self, id: u64) -> Option<Kline> {
        let idx = self.index(id)?;
        Some(self[idx])
    }

    /// 必须传递已经对齐了k线Epoch的值作为参数，数量可能少于limit
    pub fn n_ks_before(&self, to: u64, limit: usize) -> &[Kline] {
        match self.index(to) {
            None => &[],
            Some(end_idx) if end_idx + 1 > limit => {
                return &self.as_slice()[(end_idx + 1 - limit)..=end_idx]
            }
            Some(end_idx) => return &self.as_slice()[..=end_idx],
        }
    }

    /// 必须传递已经对齐了k线Epoch的值作为参数，数量可能少于limit
    pub fn n_ks_after(&self, from: u64, limit: usize) -> &[Kline] {
        match self.index(from) {
            None => &[],
            Some(start_idx) if start_idx + limit > self.len() => {
                return &self.as_slice()[start_idx..]
            }
            Some(start_idx) => return &self.as_slice()[start_idx..(start_idx + limit)],
        }
    }

    /// 可以传递没有对齐K线Epoch的参数值，返回的结果包含所有 from <= kl.id <= to 的K线
    pub fn ks_between(&self, from: u64, to: u64) -> &[Kline] {
        let start_res = self.0.binary_search_by(|kl| kl.id.cmp(&from));
        let end_res = self.0.binary_search_by(|kl| kl.id.cmp(&to));
        match (start_res, end_res) {
            (Ok(s), Ok(e)) => &self.as_slice()[s..=e],
            (Ok(s), Err(e)) => &self.as_slice()[s..e],
            (Err(s), Ok(e)) => &self.as_slice()[s..=e],
            (Err(s), Err(e)) => &self.as_slice()[s..e],
        }
    }

    pub fn push(&mut self, kl: Kline) {
        self.0.push(kl)
    }

    pub fn extend(mut self, ks: Klines) -> Self {
        for kl in ks.into_iter() {
            self.push(kl);
        }
        self
    }

    pub fn pop(&mut self) -> Option<Kline> {
        self.0.pop()
    }

    pub fn iter(&self) -> Iter<Kline> {
        self.0.iter()
    }

    // pub fn into_iter(self) -> std::vec::IntoIter<Kline> {
    //     self.0.into_iter()
    // }

    pub fn rev(&mut self) -> &mut Self {
        self.0.reverse();
        self
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn last(&self) -> Option<&Kline> {
        self.0.last()
    }

    /// 消费Klines并取前n根或后n根K线，n大于0，表示取前n根k线，n小于0，表示取后n根K线
    /// 返回的结果中，k线数量可能小于n(k线数量不足n)
    pub fn take(self, n: isize) -> Self {
        let iter = self.0.into_iter();
        let ks_iter = match n.cmp(&0isize) {
            Ordering::Equal => panic!("n should not be 0"),
            Ordering::Less => iter.rev().take(n.abs() as usize).rev().collect(),
            Ordering::Greater => iter.take(n as usize).collect(),
        };
        Self(ks_iter)
    }

    pub fn take_between(self, from: u64, to: u64) -> Self {
        let start_res = self.0.binary_search_by(|kl| kl.id.cmp(&from));
        let end_res = self.0.binary_search_by(|kl| kl.id.cmp(&to));
        let r = match (start_res, end_res) {
            (Ok(s), Ok(e)) => s..=e,         // (s, e),       //&self.as_slice()[s..=e],
            (Ok(s), Err(e)) => s..=(e - 1),  // (s, e - 1),  //&self.as_slice()[s..e],
            (Err(s), Ok(e)) => s..=e,        // (s, e),      //&self.as_slice()[s..=e],
            (Err(s), Err(e)) => s..=(e - 1), // (s, e - 1), //&self.as_slice()[s..e],
        };

        let mut ks = Self::new();
        for (idx, kl) in self.into_iter().enumerate() {
            if r.contains(&idx) {
                ks.push(kl);
            }
        }

        ks
    }

    /// 类似self.take()，但不消费而是克隆所需的n根K线
    pub fn take_clone(&self, n: isize) -> Self {
        let iter = self.0.iter();
        let ks_iter = match n.cmp(&0isize) {
            Ordering::Equal => panic!("n should not be 0"),
            Ordering::Less => iter.rev().take(n.abs() as usize).rev().cloned().collect(),
            Ordering::Greater => iter.take(n as usize).cloned().collect(),
        };
        Self(ks_iter)
    }

    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut(&Kline, &Kline) -> Ordering,
    {
        self.0.sort_unstable_by(f)
    }
}

impl ops::Deref for Klines {
    type Target = [Kline];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::Index<usize> for Klines {
    type Output = Kline;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IntoIterator for Klines {
    type Item = Kline;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Kline> for Klines {
    fn from_iter<T: IntoIterator<Item = Kline>>(iter: T) -> Self {
        let mut ks = Self::new();
        iter.into_iter().for_each(|kl| ks.push(kl));
        ks
    }
}
