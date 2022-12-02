use std::{
    io::{BufRead, BufReader},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:12345").unwrap();

    // accept connections and process them serially
    for stream in listener.incoming() {
        let s = stream.unwrap();
        println!("conn from: {}", s.peer_addr().unwrap());
        let mut stm = BufReader::new(s);
        loop {
            let mut buf = String::new();
            match stm.read_line(&mut buf) {
                Err(e) => println!("error: {}", e),
                Ok(0) => {
                    println!("closed");
                    break;
                }
                Ok(_n) => {
                    print!("readed: {}", buf);
                    buf.clear();
                }
            }
        }

        // response
        // s.write_all(b"HELLO WORLD\nhello world\nefg").unwrap();
        // s.flush().unwrap();
    }
}
