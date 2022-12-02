use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() {
    let conn = TcpStream::connect("127.0.0.1:12345").await.unwrap();
    // Framed既是Stream，也是Sink
    // Stream用于读(Framed负责将底层字节转换为Frame，然后被Stream读取)，
    // Sink用于写(Framed负责将用户层的Frame转换为底层字节，然后写入Sink)，
    // StreamExt和SinkExt提供了相关的方便的方法
    let framed = Framed::new(conn, LinesCodec::new());
    let (mut sink, mut stream) = framed.split::<String>();

    let read_task = tokio::spawn(async move {
        while let Some(read_res) = stream.next().await {
            let read_str = read_res.unwrap();
            println!("readed: {}", read_str);
        }
    });

    let write_task = tokio::spawn(async move {
        let dur = tokio::time::Duration::from_secs(1);
        loop {
            tokio::time::sleep(dur).await;
            let msg = format!(
                "{}: hello world",
                chrono::Local::now().format("%F %T")
            );

            // 方式一: feed() + flush()
            sink.feed(msg.clone()).await.unwrap();
            sink.flush().await.unwrap();

            // 方式二: send() == feed() + flush()
            sink.send(msg.clone()).await.unwrap();

            // 方式三：send_all()，一次发送一条或多条，但只允许futures::TryStream作为参数，
            // 所以要用到futures crates来构建Stream。例如：
            // let msgs = vec![Ok("hello world".to_string()), Ok("HELLO WORLD".to_string())];
            let msgs = vec!["hello world".to_string(), "HELLO WORLD".to_string()];
           
            let mut ss = futures_util::stream::iter(msgs).map(Ok);
            sink.send_all(&mut ss).await.unwrap();
        }
    });

    read_task.await.unwrap();
    write_task.await.unwrap();
}
