use serde_json::{to_vec, Value};
use std::io::Write;
use shared::mio::net::TcpStream;
use std::net::SocketAddr;
use uuid::Uuid;

/// Holds a reference to a single connected TCP client
#[derive(Debug)]
pub struct Client {
    /// Random ID of the client
    pub id: Uuid,
    /// The name of a client, if any
    pub name: Option<String>,
    /// The remote address of the client
    address: SocketAddr,
    /// The writer that is associated with the TcpStream
    writer: TcpStream,
    /// A list of channels that this client is listening to
    pub listening_to: Vec<String>,
}

impl Client {
    /// Create a new client with the given address and writer
    /// an ID will be automatically generated, and the client will not be listening to anything
    pub fn new(address: SocketAddr, writer: TcpStream) -> Client {
        Client {
            id: Uuid::new_v4(),
            name: None,
            address,
            writer,
            listening_to: Vec::new(),
        }
    }

    /// Send a given JSON message to a client
    /// This function is blocking
    pub fn send(&mut self, message: &Value) {
        // TODO: Figure out how to make this function non-blocking
        let mut bytes = to_vec(message).unwrap();
        bytes.extend(&[b'\r', b'\n']);
        self.writer.write_all(&bytes).unwrap();
    }

    /// Checks if the client is listening to the given channel
    pub fn is_listening_to(&self, action: &str) -> bool {
        ::shared::listening_to(&self.listening_to, action)
    }
}
