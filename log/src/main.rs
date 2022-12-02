use chrono::Local;
use std::io;
use tracing::*;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{self, fmt::time::FormatTime};

// tracing用法简述
//   (1).tracing可以记录结构化的日志，可以按区间(span)记录日志，例如一个函数可以作为一个区间单元，也可以自行指定何时进入(span.enter())区间单元
//   (2).tracing有TRACE DEBUG INFO WARN ERROR共5个日志级别，TRACE最详细
//   (3).tracing crate提供了最基本的核心功能：
//     - span：区间单元，具有区间的起始时间和区间的结束位置，是一个有时间跨度的区间
//             span!()创建一个span区间，span.enter()表示进入span区间，drop span的时候退出span区间
//     - event: 每一次事件，都是一条记录，也可以看作是一条日志
//             event!()记录某个指定日志级别的日志信息，event!(Level::INFO, "something happened!");
//             trace!() debug!() info!() warn!() error!()，是event!()的语法糖，可以无需再指定日志级别
//   (4).记录日志时，可以记录结构化数据，以`key=value`的方式提供和记录。例如：
//     trace!(num = 33, "hello world")，将记录为"num = 33 hello worl"
//     支持哪些格式，参考https://docs.rs/tracing/latest/tracing/index.html#recording-fields
//   (5).tracing crate自身不会记录日志，它只是发出event!()或类似宏记录的日志，
//       发出日志后，还需要通过tracing subscriber来收集
//   (6).在可执行程序(例如main函数)中，需要初始化subscriber，而在其它地方(如库或函数中)，
//       只需使用那些宏来发出日志即可。发日志和收集记录日志分开，使得日志的处理逻辑非常简洁
//   (7).初始化subscriber的时候，可筛选收集到的日志(例如指定过滤哪些级别的日志)、格式化收集到的日志(例如修改时间格式)、指定日志的输出位置，等等
//   (8).默认清空下，subscribe的默认输出位置是标准输出，但可以在初始化时改变目标位置。如果需要写入文件，可使用tracing_appender crate

struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%FT%T%.3f"))
    }
}

// 通过instrument属性，直接让整个函数或方法进入span区间，且适用于异步函数async fn fn_name(){}
// 参考：https://docs.rs/tracing/latest/tracing/attr.instrument.html
// #[tracing::instrument(level = "info")]
#[instrument]
fn test_trace(n: i32) {
    // #[instrument]属性表示函数整体在一个span区间内，因此函数内的每一个event信息中都会额外带有函数参数
    // 在函数中，只需发出日志即可
    event!(Level::TRACE, answer = 42, "trace2: test_trace");
    trace!(answer = 42, "trace1: test_trace");
    info!(answer = 42, "info1: test_trace");
}

// 在可执行程序中，需初始化tracing subscriber来收集、筛选并按照指定格式来记录日志
fn main() {
    // // 直接初始化，采用默认的Subscriber，默认只输出INFO、WARN、ERROR级别的日志
    // // tracing_subscriber::fmt::init();

    // // 使用tracing_appender，指定日志的输出目标位置
    // // 参考: https://docs.rs/tracing-appender/0.2.0/tracing_appender/
    // let file_appender = tracing_appender::rolling::daily("/tmp", "tracing.log");
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // // 设置日志输出时的格式，例如，是否包含日志级别、是否包含日志来源位置、设置日志的时间格式
    // // 参考: https://docs.rs/tracing-subscriber/0.3.3/tracing_subscriber/fmt/struct.SubscriberBuilder.html#method.with_timer
    // let format = tracing_subscriber::fmt::format()
    //     .with_level(true)
    //     .with_target(true)
    //     .with_timer(LocalTimer);

    // // 初始化并设置日志格式(定制和筛选日志)
    // tracing_subscriber::fmt()
    //     .with_max_level(Level::TRACE)
    //     .with_writer(io::stdout) // 写入标准输出
    //     .with_writer(non_blocking) // 写入文件，将覆盖上面的标准输出
    //     .event_format(format)
    //     .init();

    // test_trace(33);
    // trace!("tracing-trace");
    // debug!("tracing-debug");
    // info!("tracing-info");
    // warn!("tracing-warn");
    // error!("tracing-error");

    // 先创建一个SubScriber，准备作为默认的全局SubScriber
    let default_logger = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .finish();

    // 这段代码不记录任何日志，因为还未开启任何SubScriber
    {
        info!("nothing will log");
    }

    // 从此开始，将default_logger设置为全局SubScriber
    default_logger.init();

    // 创建一个只记录ERROR级别的SubScriber
    let tmp_logger = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(Level::ERROR)
        .finish();
    // 使用with_default，可将某段代码使用指定的SubScriber而非全局的SubScriber进行日志记录
    tracing::subscriber::with_default(tmp_logger, || {
        error!("log with tmp_logger, only log ERROR logs");
    });
    info!("log with Global logger");
}
