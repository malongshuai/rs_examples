use std::io::Error;

use futures::StreamExt;
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::low_level;
use signal_hook_tokio::Signals;

async fn handle_signals(mut signals: Signals) {
    let mut n = 0;
    while let Some(signal) = signals.next().await {
        // 信号(数值)转换为型号名称
        let sig_str = low_level::signal_name(signal).unwrap();
        match signal {
            SIGHUP => {
                println!("receive: {}", sig_str);
            }
            SIGTERM | SIGINT => {
                if n == 10 {
                    println!("exit {} {}", n, sig_str);
                    low_level::exit(1);
                }
                println!("receive: {} {}", n, sig_str);
                n += 1;
            }
            _ => {}
        }
    }
}

#[tokio::main]
pub async fn tokio_handle() -> Result<(), Error> {
    let signals = Signals::new(&[SIGTERM, SIGINT])?;
    let handle = signals.handle();

    let signals_task = tokio::spawn(handle_signals(signals));

    std::thread::sleep(std::time::Duration::from_secs(100));

    // Terminate the signal stream.
    handle.close();
    signals_task.await?;

    Ok(())
}
