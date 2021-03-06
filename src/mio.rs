use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Registry, Token};
use slab::Slab;

use std::io::{self, Read, Write};

const SERVER: Token = Token(usize::MAX);

const ADDR: &str = "0.0.0.0:12321";

struct Session {
    connection: TcpStream,
    buffer: Vec<u8>,
}

fn main() -> io::Result<()> {
    println!("starting pingserver_mio");
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    let addr = ADDR.parse().unwrap();
    let mut server = TcpListener::bind(addr)?;

    poll.registry()
        .register(&mut server, SERVER, Interest::READABLE)?;

    let mut connections = Slab::with_capacity(1024);

    println!("listening on: {}", ADDR);

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER => loop {
                    let (mut connection, _address) = match server.accept() {
                        Ok((connection, address)) => (connection, address),
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    };

                    let session_entry = connections.vacant_entry();
                    let token = Token(session_entry.key());

                    // let token = next(&mut unique_token);
                    poll.registry()
                        .register(&mut connection, token, Interest::READABLE)?;

                    let session = Session {
                        connection,
                        buffer: vec![0; 1024],
                    };

                    session_entry.insert(session);

                    // connections.insert(token, session);
                },
                token => {
                    let done = if let Some(connection) = connections.get_mut(token.0) {
                        handle_connection_event(poll.registry(), connection, event)?
                    } else {
                        false
                    };
                    if done {
                        connections.remove(token.0);
                    }
                }
            }
        }
    }
}

/// Returns `true` if the connection is done.
fn handle_connection_event(
    _registry: &Registry,
    session: &mut Session,
    event: &Event,
) -> io::Result<bool> {
    if event.is_readable() {
        let mut connection_closed = false;
        let mut bytes_read = 0;

        loop {
            match session.connection.read(&mut session.buffer) {
                Ok(0) => {
                    connection_closed = true;
                    break;
                }
                Ok(n) => {
                    bytes_read += n;
                    if bytes_read == session.buffer.len() {
                        session.buffer.resize(session.buffer.len() + 1024, 0);
                    }
                }
                Err(ref err) if would_block(err) => break,
                Err(ref err) if interrupted(err) => continue,
                Err(err) => return Err(err),
            }
        }

        if &session.buffer[0..6] == b"PING\r\n" || &session.buffer[0..6] == b"ping\r\n" {
            session.connection.write_all(b"PONG\r\n")?;
            session.buffer.write_all(&[0, 0, 0, 0, 0, 0])?;
        }

        if connection_closed {
            println!("Connection closed");
            return Ok(true);
        }
    }

    Ok(false)
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
