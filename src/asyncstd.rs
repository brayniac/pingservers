use smol::io::{Error, ErrorKind};

use async_std::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
enum Void {}

const ADDR: &str = "127.0.0.1:12321";

pub fn main() -> Result<()> {
    task::block_on(accept_loop(ADDR))
}

async fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        spawn_and_log_error(connection_loop(stream));
    }

    Ok(())
}

async fn connection_loop(mut stream: TcpStream) -> Result<()> {
    let mut buffer = vec![0; 1024];

    loop {
        let bytes = stream.read(&mut buffer).await.expect("failed to read");

        if bytes == 0 {
            return Err(Box::new(Error::new(ErrorKind::Other, "disconnect")));
        }

        if &buffer[0..6] == b"PING\r\n" || &buffer[0..6] == b"ping\r\n" {
            stream
                .write_all(b"PONG\r\n")
                .await
                .expect("failed to write response to socket");
            buffer
                .write_all(&[0, 0, 0, 0, 0, 0])
                .await
                .expect("failed to clear buffer");
        }
    }
}

fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    })
}
