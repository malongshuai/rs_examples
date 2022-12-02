#![allow(clippy::explicit_counter_loop)]
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
    low_level,
};
use std::{error::Error, thread, time::Duration};

pub fn easy_use() -> Result<(), Box<dyn Error>> {
    let mut signals = Signals::new(&[SIGINT, SIGTERM])?;
    let handle = signals.handle();

    thread::spawn(move || {
        let mut n = 0;
        for sig in signals.forever() {
            // 信号(数值)转换为型号名称
            let sig_str = low_level::signal_name(sig).unwrap();
            if n == 10 {
                println!("exit {} {}", n, sig_str);
                low_level::exit(1);
            }
            println!("Received signal {} {}", n, sig_str);
            n += 1;
        }
    });

    thread::sleep(Duration::from_secs(100));

    handle.close();
    Ok(())
}
