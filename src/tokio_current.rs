use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use std::error::Error;

const ADDR: &str = "0.0.0.0:12321";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("starting pingserver_tokio_current");

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
