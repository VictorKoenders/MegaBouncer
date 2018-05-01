use futures::sync::mpsc::Sender;
use futures::{Future, Sink};
use serde_json::Value;
use server::ServerMessage;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_io::io::WriteHalf;

#[derive(Clone)]
pub struct ServerHandle {
    sender: Sender<ServerMessage>,
}

impl ServerHandle {
    pub fn new(sender: Sender<ServerMessage>) -> ServerHandle {
        ServerHandle { sender }
    }

    pub fn client_connected(
        &mut self,
        address: SocketAddr,
        writer: WriteHalf<TcpStream>,
    ) -> Box<Future<Item = (), Error = ()> + Send> {
        Box::new(
            self.sender
                .clone()
                .send(ServerMessage::ClientConnected(address, writer))
                .map_err(|e| {
                    println!("Could not send error to server: {:?}", e);
                })
                .map(|_| {}),
        )
    }
    pub fn message_received(
        &mut self,
        address: SocketAddr,
        value: Value,
    ) -> Box<Future<Item = (), Error = ()> + Send> {
        Box::new(
            self.sender
                .clone()
                .send(ServerMessage::Message(address, value))
                .map_err(|e| {
                    println!("Could not send error to server: {:?}", e);
                })
                .map(|_| {}),
        )
    }
}
