use client::Client;
use futures::sync::mpsc::{channel, Receiver};
use futures::{Async, Future, Stream};
use serde_json::Value;
use serverhandle::ServerHandle;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_io::io::WriteHalf;

#[derive(Debug)]
pub enum ServerMessage {
    ClientConnected(SocketAddr, WriteHalf<TcpStream>),
    Message(SocketAddr, Value),
}

#[derive(Debug)]
pub struct Server {
    receiver: Receiver<ServerMessage>,
}
#[derive(Default, Debug)]
struct ServerState {
    clients: Vec<Client>,
}

impl Server {
    pub fn new() -> (ServerHandle, Server) {
        let (sender, receiver) = channel(10);

        (ServerHandle::new(sender), Server { receiver })
    }

    pub fn start(self) -> Box<Future<Item = (), Error = ()> + Send + 'static> {
        use std::sync::Arc;
        use std::sync::Mutex;
        let state = Arc::new(Mutex::new(ServerState {
            clients: Vec::new(),
        }));
        Box::new(self.receiver.for_each(move |message| {
            let state = state.clone();
            println!("message: {:?}", message);
            println!("self: {:?}", state);
            Ok(())
        }))
    }
}

impl Future for Server {
    type Item = ();
    type Error = io::Error;
    fn poll(&mut self) -> Result<Async<()>, Self::Error> {
        Ok(Async::NotReady)
    }
}
