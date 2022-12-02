use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;

/// TERM_SIGNALS包含了所有约定俗成的退出信号([SIGTERM, SIGQUIT, SIGINT])

/// 1.如果将register_conditional_shutdown放在register后面，将在第一次接收信号时终止
pub fn flag_register1() {
    let term = Arc::new(AtomicBool::new(false));

    for sig in TERM_SIGNALS {
        {
            // 注册信号，当接收到该信号时，term会被设置为true
            flag::register(*sig, term.clone()).unwrap();

            // 当term值为true时，将以退出状态码1终止程序
            flag::register_conditional_shutdown(*sig, 1, term.clone()).unwrap();
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(100));
}

/// 2.如果将register_conditional_shutdown放在register前面，将在第二次接收信号时终止
pub fn flag_register2() {
    let term = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        {
            // 当term值为true时，将以退出状态码1终止程序
            flag::register_conditional_shutdown(*sig, 1, term.clone()).unwrap();

            // 注册信号，当接收到该信号时，term会被设置为true
            flag::register(*sig, term.clone()).unwrap();
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(100));
}
