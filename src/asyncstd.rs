
use smol::io::{Error, ErrorKind};




use async_std::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
// type Sender<T> = mpsc::UnboundedSender<T>;
// type Receiver<T> = mpsc::UnboundedReceiver<T>;

#[derive(Debug)]
enum Void {}

const ADDR: &str = "127.0.0.1:12321";

pub fn main() -> Result<()> {
    task::block_on(accept_loop(ADDR))
}

async fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;

    // let (broker_sender, broker_receiver) = mpsc::unbounded();
    // let broker = task::spawn(broker_loop(broker_receiver));
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        spawn_and_log_error(connection_loop(stream));
    }
    // drop(broker_sender);
    // broker.await;
    Ok(())
}

async fn connection_loop(mut stream: TcpStream) -> Result<()> {
    let mut buffer = vec![0; 1024];
    // let stream = Arc::new(stream);

    loop {
        let bytes = stream.read(&mut buffer).await.expect("failed to read");

        if bytes == 0 {
            return Err(Box::new(Error::new(ErrorKind::Other, "disconnect")));
        }

        if &buffer[0..6] == b"PING\r\n" || &buffer[0..6] == b"ping\r\n" {
            stream.write_all(b"PONG\r\n").await.expect("failed to write response to socket");
            buffer.write_all(&[0,0,0,0,0,0]).await.expect("failed to clear buffer");
        }
    }
    
}

//     let name = match lines.next().await {
//         None => return Err("peer disconnected immediately".into()),
//         Some(line) => line?,
//     };
//     let (_shutdown_sender, shutdown_receiver) = mpsc::unbounded::<Void>();
//     broker
//         .send(Event::NewPeer {
//             name: name.clone(),
//             stream: Arc::clone(&stream),
//             shutdown: shutdown_receiver,
//         })
//         .await
//         .unwrap();

//     while let Some(line) = lines.next().await {
//         let line = line?;
//         let (dest, msg) = match line.find(':') {
//             None => continue,
//             Some(idx) => (&line[..idx], line[idx + 1..].trim()),
//         };
//         let dest: Vec<String> = dest
//             .split(',')
//             .map(|name| name.trim().to_string())
//             .collect();
//         let msg: String = msg.trim().to_string();

//         broker
//             .send(Event::Message {
//                 from: name.clone(),
//                 to: dest,
//                 msg,
//             })
//             .await
//             .unwrap();
//     }

//     Ok(())
// }

// async fn connection_writer_loop(
//     messages: &mut Receiver<String>,
//     stream: Arc<TcpStream>,
//     mut shutdown: Receiver<Void>,
// ) -> Result<()> {
//     let mut stream = &*stream;
//     loop {
//         select! {
//             msg = messages.next().fuse() => match msg {
//                 Some(msg) => stream.write_all(msg.as_bytes()).await?,
//                 None => break,
//             },
//             void = shutdown.next().fuse() => match void {
//                 Some(void) => match void {},
//                 None => break,
//             }
//         }
//     }
//     Ok(())
// }

// #[derive(Debug)]
// enum Event {
//     NewPeer {
//         name: String,
//         stream: Arc<TcpStream>,
//         shutdown: Receiver<Void>,
//     },
//     Message {
//         from: String,
//         to: Vec<String>,
//         msg: String,
//     },
// }

// async fn broker_loop(mut events: Receiver<Event>) {
//     let (disconnect_sender, mut disconnect_receiver) =
//         mpsc::unbounded::<(String, Receiver<String>)>();
//     let mut peers: HashMap<String, Sender<String>> = HashMap::new();

//     loop {
//         let event = select! {
//             event = events.next().fuse() => match event {
//                 None => break,
//                 Some(event) => event,
//             },
//             disconnect = disconnect_receiver.next().fuse() => {
//                 let (name, _pending_messages) = disconnect.unwrap();
//                 assert!(peers.remove(&name).is_some());
//                 continue;
//             },
//         };
//         match event {
//             Event::Message { from, to, msg } => {
//                 for addr in to {
//                     if let Some(peer) = peers.get_mut(&addr) {
//                         let msg = format!("from {}: {}\n", from, msg);
//                         peer.send(msg).await.unwrap();
//                     }
//                 }
//             }
//             Event::NewPeer {
//                 name,
//                 stream,
//                 shutdown,
//             } => match peers.entry(name.clone()) {
//                 Entry::Occupied(..) => (),
//                 Entry::Vacant(entry) => {
//                     let (client_sender, mut client_receiver) = mpsc::unbounded();
//                     entry.insert(client_sender);
//                     let mut disconnect_sender = disconnect_sender.clone();
//                     spawn_and_log_error(async move {
//                         let res =
//                             connection_writer_loop(&mut client_receiver, stream, shutdown).await;
//                         disconnect_sender
//                             .send((name, client_receiver))
//                             .await
//                             .unwrap();
//                         res
//                     });
//                 }
//             },
//         }
//     }
//     drop(peers);
//     drop(disconnect_sender);
//     while let Some((_name, _pending_messages)) = disconnect_receiver.next().await {}
// }

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
