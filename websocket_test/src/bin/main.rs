use flate2::bufread::GzDecoder;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::{borrow::Cow, io::Read};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
    MaybeTlsStream, WebSocketStream,
};
use tracing::*;
use tracing_subscriber::fmt::time::FormatTime;

const URL: &str = "wss://api.huobi.pro/ws";
const BIAN_URL: &str = "wss://stream.binance.com:9443";
// while let Some(msg) = ws_stream.next().await {
//     let msg = msg?;
//     if msg.is_text() || msg.is_binary() {
//         ws_stream.send(msg).await?;
//     }
// }

struct WrapWebSocket {
    conn: WebSocketStream<MaybeTlsStream<TcpStream>>,
    extra: Vec<String>,
}

impl WrapWebSocket {
    async fn new() {}
}

fn handle_data(str: String) {
    let x: ResponseMessage = serde_json::from_str(&str).unwrap();
    println!("handled: -------------------------");
    println!("{:?}", x);
}

#[allow(dead_code)]
#[tracing::instrument(skip_all)]
async fn req_bian(symbols: Vec<&str>, channel: &str) {
    let streams = symbols
        .iter()
        .map(|sym| format!("{}@{}", sym, channel))
        .collect::<Vec<String>>()
        .join("/");

    let url = format!("{}/stream?streams={}", BIAN_URL, streams);

    loop {
        // let url = url.clone();
        let (mut ws_stream, _response) = connect_async(&url).await.unwrap();
        debug!("connect: {}", url);

        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(msg) => match msg {
                    Message::Text(data) => {
                        debug!("text: {}", data);
                    }
                    Message::Ping(_) => {
                        let pong = Message::Pong(vec![]);
                        ws_stream.send(pong).await.unwrap();
                    }
                    Message::Close(Some(data)) => {
                        debug!("closed reason: {}", data.reason);
                    }
                    _ => {}
                },
                Err(_) => {
                    debug!("closed");
                }
            }
        }
    }
}

#[allow(dead_code)]
#[tracing::instrument]
async fn req_huobi() {
    let (mut ws_stream, _response) = connect_async(URL).await.unwrap();
    debug!("connected");

    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(msg) => match msg {
                Message::Binary(data) => {
                    let mut gz = GzDecoder::new(&data[..]);
                    let mut str = String::new();
                    gz.read_to_string(&mut str).unwrap();
                    debug!("msgs: {}", str);
                    handle_data(str);
                }
                Message::Text(data) => {
                    debug!("text: {}", data);
                }
                Message::Close(Some(data)) => {
                    debug!("closed reason: {}", data.reason);
                }
                _ => {}
            },
            Err(_) => {
                debug!("closed");
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ResponseMessage {
    Ping(Ping),
    ReqKlines(ReqKlines),
    TickKline(TickKline),
    OtherStatus(OtherStatus),
}

#[derive(Debug, Deserialize)]
struct Ping {
    ping: u64,
}

impl Ping {
    fn pong(&self) -> String {
        format!(r#"{{"pong":{}}}"#, self.ping)
    }
}

#[derive(Debug, Deserialize)]
struct ReqKlines {
    id: String,     // symbol name
    rep: String,    // channel, "market.btcusdt.kline.1min"
    status: String, // "ok" or "error"
    ts: u64,
    data: Vec<Kline>,
}

#[derive(Debug, Deserialize)]
struct TickKline {
    ch: String, // channel, "market.btcusdt.kline.1min"
    ts: u64,
    tick: Kline,
}

#[derive(Debug, Deserialize)]
struct Kline {
    id: u64,
    close: f64,
    open: f64,
    high: f64,
    low: f64,
    amount: f64,
    vol: f64,
    count: u64,
}

#[derive(Debug, Deserialize)]
struct OtherStatus {
    status: String, // "ok" or "error"
}

struct LocalTimer;
impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", chrono::Local::now().format("%FT%T%.6f"))
    }
}

#[tokio::main]
async fn main() {
    let appender = tracing_appender::rolling::daily("/mnt/g/桌面", "bian.log");
    let (log_writer, _guard) = tracing_appender::non_blocking(appender);

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_ansi(false)
        .with_timer(LocalTimer)
        .with_writer(log_writer)
        .init();

    let symbols = vec![
        "1inchusdt",
        "aaveusdt",
        "acmusdt",
        "adausdt",
        "agldusdt",
        "aionusdt",
        "akrousdt",
        "algousdt",
        "aliceusdt",
        "alpacausdt",
        "alphausdt",
        "ankrusdt",
        "antusdt",
        "ardrusdt",
        "arpausdt",
        "arusdt",
        "asrusdt",
        "atausdt",
        "atmusdt",
        "atomusdt",
        "audiousdt",
        "autousdt",
        "avausdt",
        "avaxusdt",
        "axsusdt",
        "badgerusdt",
        "bakeusdt",
        "balusdt",
        "bandusdt",
        "barusdt",
        "batusdt",
        "bchusdt",
        "beamusdt",
        "belusdt",
        "betausdt",
        "blzusdt",
        "bnbusdt",
        "bntusdt",
        "bondusdt",
        "btcstusdt",
        "btcusdt",
        "btgusdt",
        "btsusdt",
        "bttusdt",
        "burgerusdt",
        "bzrxusdt",
        "c98usdt",
        "cakeusdt",
        "celousdt",
        "celrusdt",
    ];
    let channel = "kline_1m";
    // req_bian(symbols, channel).await
    req_bian(vec!["dogeusdt"], channel).await
}

#[cfg(test)]
mod test {
    use super::handle_data;

    #[test]
    fn test() {
        let ping_str = r#"{"ping": 12345678}"#;
        handle_data(ping_str.to_string());

        let req_klines = r##"
      {
      "id": "btcusdt",
      "rep": "market.btcusdt.kline.1min",
      "status": "ok",
      "ts": 1616681203184,
      "data": [
        {
          "amount": 47.58869897859039,
          "close": 50989.63,
          "count": 1327,
          "high": 51129.91,
          "id": 1616677620,
          "low": 50986,
          "open": 51125,
          "vol": 2430238.6246752427
        },
        { 
          "amount": 48,
          "close": 61213.222,
          "count": 1327,
          "high": 51129.91,
          "id": 1616677620,
          "low": 50986,
          "open": 51125,
          "vol": 2430238.6246752427
        }
      ]
    }
  "##;
        handle_data(req_klines.to_string());

        let tick_kline = r##"
    {
      "ch": "market.btcusdt.kline.1min",
      "ts": 1489474082831,
      "tick": {
        "id": 1489464480,
        "amount": 49527.52,
        "count": 4,
        "open": 0.001161,
        "close": 0.001163,
        "low": 0.001157,
        "high": 0.001163,
        "vol": 57.5213654
      }
    }
  "##;
        handle_data(tick_kline.to_string());

        let status = r##"
  {
    "id": 18293791278,
    "status": "error",
    "subbed": "market.btcusdt.kline.1min",
    "ts": 1489474081631
  }
  "##;
        handle_data(status.to_string());
    }
}
