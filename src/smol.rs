use smol::io::AsyncReadExt;
use smol::io::AsyncWriteExt;
use std::net::TcpListener;

use smol::{io, Async};

fn main() -> io::Result<()> {
    println!("starting pingserver_smol");
    smol::block_on(async {
        // Create a listener.
        let listener = Async::<TcpListener>::bind(([0, 0, 0, 0], 12321))?;
        println!("Listening on {}", listener.get_ref().local_addr()?);

        // Accept clients in a loop.
        loop {
            let (mut socket, peer_addr) = listener.accept().await?;
            println!("Accepted client: {}", peer_addr);

            // Spawn a task that echoes messages from the client back to it.
            smol::spawn(async move {
                let mut buf = vec![0; 1024];

                // In a loop, read data from the socket and write the data back.
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
            })
            .detach();
        }
    })
}
