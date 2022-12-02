#![allow(unused_imports)]

use std::collections::HashMap;
use std::ops::Deref;
use std::time::{self, Instant};
use std::{env, error, thread};

use chrono::{FixedOffset, TimeZone, Timelike};
use clia_local_time::LocalTime;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use itertools::Itertools;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use ::time::macros::format_description;
use serde_json::{json, Value};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct PDust1 {
    /// 参数格式 asset=BTC&asset=USDT&asset=ETH
    /// 因此包一个option
    asset: Vec<String>,
}
impl PDust1 {
    /// 参数格式 asset=BTC&asset=USDT&asset=ETH
    pub fn new(assets: &[&str]) -> Self {
        let mut s = Vec::new();
        for asset in assets {
            let asset = asset.to_uppercase();
            s.push(asset);
        }
        Self { asset: s }
    }
}

#[derive(Debug)]
pub struct PDust {
    /// 参数格式 asset=BTC&asset=USDT&asset=ETH
    /// 因此包一个option
    asset: Vec<String>,
}

impl PDust {
    /// 参数格式 asset=BTC&asset=USDT&asset=ETH
    pub fn new(assets: &[&str]) -> Self {
        let mut s = Vec::new();
        for asset in assets {
            let asset = asset.to_uppercase();
            s.push(asset);
        }
        Self { asset: s }
    }
}

impl Serialize for PDust {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut dust = serializer.serialize_struct("PDust", self.asset.len())?;
        for asset in &self.asset {
            dust.serialize_field("asset", asset)?;
        }
        dust.end()
    }
}

#[tokio::main]
async fn main() {
    let n = PDust::new(&["btc", "eth", "usdt"]);
    let res = serde_url_params::to_string(&n).unwrap();
    let res1 = serde_url_params::to_string(&n).unwrap();

    println!("{:?}", res);
    println!("{:?}", res1);
    // assert_eq!("asset%3DBTC%26asset%3DUSDT%26asset%3DETH", res)
    println!("{:?}", serde_json::to_string(&n).unwrap());


    tracing_subscriber::fmt().without_time().init();
    tracing::error!("hellow rold");
}
