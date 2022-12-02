#![allow(dead_code)]

mod easy_use;
mod flag_register;
mod tokio_sig_handle;

#[tokio::main]
async fn main() {
    // 1.最简单的用法
    easy_use::easy_use().unwrap();

    // 2.通过flag::register注册信号处理程序，当接收信号时，修改布尔值
    // flag_register::flag_register1();
    // flag_register::flag_register2();

    // 3.tokio中注册信号处理程序
    // tokio_sig_handle::tokio_handle().await;
}
