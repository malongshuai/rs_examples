use bytes::BytesMut;
use clia_local_time::LocalTime;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    sync::{Arc, RwLock},
    thread,
};
use time::macros::format_description;
use tracing::info;

#[derive(Clone)]
struct Rst {
    stream: Arc<RwLock<TcpStream>>,
}

impl Rst {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream: Arc::new(RwLock::new(stream)),
        }
    }

    fn send_message(&self, msg: &str) {
        let mut writer = self.stream.write().unwrap();
        let n = writer.write(msg.as_bytes()).unwrap();
        writer.flush().unwrap();
        info!("write {} bytes", n);
    }

    fn read_message(&self) -> Result<String, io::Error> {
        let mut reader = self.stream.write().unwrap();
        let mut buf = BytesMut::new();
        let mut str = String::new();
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                info!("read 0 bytes");
                continue;
            }

            str = String::from_utf8(buf.split_to(n).to_vec()).unwrap();
            break;
        }
        Ok(str)
    }
}

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

    let mut stream = TcpStream::connect("127.0.0.1:12345").unwrap();
    // let rst = Rst::new(stream);

    // let mut tasks = vec![];
    // let rst1 = rst.clone();
    // let t1 = thread::spawn(move || rst1.send_message("hello world"));

    // let rst2 = rst.clone();
    // let t2 = thread::spawn(move || {
    //     let msg = rst2.read_message();
    //     info!("read msg: {:?}", msg);
    // });

    // tasks.push(t1);
    // tasks.push(t2);

    // for task in tasks {
    //     task.join().unwrap();
    // }

    let n = stream.write(b"hello world 1").unwrap();
    stream.flush().unwrap();
    info!("write 1 {} bytes", n);
    std::thread::sleep(std::time::Duration::from_secs(1));
    let n = stream.write(b"hello world 2").unwrap();
    stream.flush().unwrap();
    info!("write 2 {} bytes", n);

    let mut buf = [0; 512];
    // let mut str = String::new();
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            info!("read 0 bytes, buf len: {}", buf.len());
            break;
        }

        info!("read {} bytes, buf len: {}", n, buf.len());

        // str = String::from_utf8(buf.to_vec()).unwrap();
        // break;
    }
    // info!("read msg: {:?}", str);
}
