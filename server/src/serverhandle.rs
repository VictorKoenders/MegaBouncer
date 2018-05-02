use serde_json::Value;
use server::ServerMessage;
use shared::futures::sync::mpsc::Sender;
use shared::futures::{Future, Sink};
use shared::tokio::net::TcpStream;
use shared::tokio_io::io::WriteHalf;
use shared::EmptyFuture;
use std::net::SocketAddr;

/// A lightweight handle that allows futures to send messages to the central server
#[derive(Clone)]
pub struct ServerHandle {
    sender: Sender<ServerMessage>,
}

impl ServerHandle {
    /// Create a new server handle
    pub fn new(sender: Sender<ServerMessage>) -> ServerHandle {
        ServerHandle { sender }
    }

    /// Send a "ClientConnected" message to the central server
    pub fn client_connected(
        &mut self,
        address: SocketAddr,
        writer: WriteHalf<TcpStream>,
    ) -> EmptyFuture {
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

    /// Send a "ClientDisconnected" message to the central server
    pub fn client_disconnected(&mut self, address: SocketAddr) -> EmptyFuture {
        Box::new(
            self.sender
                .clone()
                .send(ServerMessage::ClientDisconnected(address))
                .map_err(|e| {
                    println!("Could not send error to server: {:?}", e);
                })
                .map(|_| {}),
        )
    }
    /// Send a "Message" message to the central server
    pub fn message_received(&mut self, address: SocketAddr, value: Value) -> EmptyFuture {
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
