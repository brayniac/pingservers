use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::runtime::Builder;

use std::error::Error;

const ADDR: &str = "127.0.0.1:12321";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("starting pingserver_tokio");
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("ping_worker")
        .thread_stack_size(3 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap();

    let _guard = runtime.enter();

    let listener = TcpListener::bind(&ADDR).await?;
    println!("Listening on: {}", ADDR);

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                if buf.len() < 6 {
                    continue;
                }

                if &buf[0..6] == b"PING\r\n" || &buf[0..6] == b"ping\r\n" {
                    socket
                        .write_all(b"PONG\r\n")
                        .await
                        .expect("failed to write response to socket");
                    buf.write_all(&[0, 0, 0, 0, 0, 0])
                        .await
                        .expect("failed to clear buffer");
                }
            }
        });
    }
}
