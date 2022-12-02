use clia_local_time::LocalTime;
use std::{io::Read, net::TcpListener};
use time::macros::format_description;
use tracing::{error, info, warn};

fn main() {
    let local_time_fmt =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6]");
    let log_local_timer = LocalTime::with_timezone(local_time_fmt, (8, 0, 0));
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_timer(log_local_timer)
        .init();

    let listener = TcpListener::bind("127.0.0.1:12345").unwrap();

    // accept connections and process them serially
    for stream in listener.incoming() {
        let mut s = stream.unwrap();
        info!("conn from: {}", s.peer_addr().unwrap());
        // let mut stm = BufReader::new(s);
        let mut buf = [0; 8];
        loop {
            // let mut buf = String::new();
            match s.read(&mut buf) {
                Err(e) => error!("error: {}", e),
                // Ok(0) => {
                //     warn!("peer closed, buf len: {}", buf.len());
                //     // break;
                // }
                Ok(n) => {
                    if n == 0 {
                        warn!("peer closed {}, buf len: {}", n, buf.len());
                        break;
                    }
                    info!("readed bytes: {},  {}", n, buf.len());
                    // buf.clear();
                }
            }
        }

        // response
        // s.write_all(b"HELLO WORLD\nhello world\nefg").unwrap();
        // s.flush().unwrap();
    }
}
