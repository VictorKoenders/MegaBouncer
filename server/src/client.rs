use serde_json::{to_vec, Value};
use shared::mio::Event;
use shared::mio::net::TcpStream;
use std::net::SocketAddr;
use uuid::Uuid;
use std::io::{Result, Write, Read, ErrorKind};

/// Holds a reference to a single connected TCP client
#[derive(Debug)]
pub struct Client {
    /// Random ID of the client
    pub id: Uuid,
    /// The name of a client, if any
    pub name: Option<String>,
    /// The remote address of the client
    address: SocketAddr,
    stream: TcpStream,
    write_buff: Vec<u8>,
    read_buff: Vec<u8>,
    is_writable: bool,
    is_readable: bool,
    /// The writer that is associated with the TcpStream
    /// A list of channels that this client is listening to
    pub listening_to: Vec<String>,
}

impl Client {
    /// Create a new client with the given address and writer
    /// an ID will be automatically generated, and the client will not be listening to anything
    pub fn new(address: SocketAddr, stream: TcpStream) -> Client {
        Client {
            id: Uuid::new_v4(),
            name: None,
            address,
            stream,
            write_buff: Vec::new(),
            read_buff: Vec::new(),
            is_writable: false,
            is_readable: false,
            listening_to: Vec::new(),
        }
    }

    /// Send a given JSON message to a client
    /// This function is blocking
    pub fn send(&mut self, message: &Value) -> Result<()> {
        println!("Writing {:?}", message);
        let bytes = to_vec(message).unwrap();
        self.write_buff.extend(bytes);
        self.write_buff.extend(&[b'\r', b'\n']);

        if self.is_writable {
            self.process_write()?;
        }
        Ok(())
    }

    fn process_write(&mut self) -> Result<()> {
        loop {
            if self.write_buff.len() == 0 {
                return Ok(());
            }
            return match self.stream.write(&self.write_buff) {
                Ok(n) => {
                    self.write_buff.drain(..n);
                    if self.write_buff.len() == 0 {
                        return Ok(());
                    }
                    continue;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }
    }

    fn identify(&mut self, json: &Value) -> Option<ClientUpdate> {
        if let Some(name) = json.as_object().and_then(|o| o.get("name")).and_then(|o| o.as_str()) {
            self.name = Some(name.to_string());
            Some(ClientUpdate::Identified(name.to_string()))
        } else if let Err(e) = self.send(&::server::make_error("Missing required field 'name'")) {
            self.print_error(e);
            Some(ClientUpdate::Disconnect)
        } else {
            None
        }
    }

    fn register_listener(&mut self, json: &Value) -> Option<ClientUpdate> {
        if let Some(channel) = json.as_object().and_then(|o| o.get("channel")).and_then(|o| o.as_str()) {
            if let Some(name) = self.name.as_ref() {
                self.listening_to.push(channel.to_string());
                return Some(ClientUpdate::Broadcast(::server::make_client_listening_to(name, channel, &self.id)));
            }
            if let Err(e) = self.send(&::server::make_error("Not identified")) {
                self.print_error(e);
                Some(ClientUpdate::Disconnect)
            } else {
                None
            }
        } else if let Err(e) = self.send(&::server::make_error("Missing required field 'name'")) {
            self.print_error(e);
            Some(ClientUpdate::Disconnect)
        } else {
            None
        }
    }

    fn handle_line(&mut self, line: &str) -> Option<ClientUpdate> {
        let json: Value = match ::serde_json::from_str(line) {
            Ok(j) => j,
            Err(e) => {
                self.print_error(e);
                return Some(ClientUpdate::Disconnect);
            }
        };
        let action = json
            .as_object()
            .and_then(|o| o.get("action"))
            .and_then(|s| s.as_str());
        if let Some("node.identify") = action {
            self.identify(&json)
        } else if self.name.is_none() {
            if let Err(e) = self.send(&::server::make_error("Not identified")) {
                self.print_error(e);
                Some(ClientUpdate::Disconnect)
            } else {
                None
            }
        } else {
            match action {
                Some("node.listener.register") => self.register_listener(&json),
                Some("node.list") => Some(ClientUpdate::ListNodes),
                Some(_) => Some(ClientUpdate::Broadcast(json.clone())),
                None => {
                    if let Err(e) = self.send(&::server::make_error("Missing required field 'action'")) {
                        self.print_error(e);
                        Some(ClientUpdate::Disconnect)
                    } else {
                        None
                    }
                }
            }
        }
    }

    fn process_read(&mut self) -> Result<Vec<ClientUpdate>> {
        let mut updates = Vec::new();
        let mut buff = [0u8; 1024];
        let mut did_read = false;
        'outer: loop {
            let result = self.stream.read(&mut buff);
            match result {
                Ok(0) => {
                    if !did_read {
                        updates.push(ClientUpdate::Disconnect);
                    }
                    break 'outer;
                }
                Ok(n) => {
                    did_read = true;
                    self.read_buff.extend(&buff[..n]);
                    while let Some(index) = self.read_buff.iter().position(|c| *c == b'\n') {
                        let line = self.read_buff.drain(..index + 1).take(index - 1).collect::<Vec<_>>();
                        let line = match ::std::str::from_utf8(&line) {
                            Ok(l) => l,
                            Err(e) => {
                                self.print_error(e);
                                updates.push(ClientUpdate::Disconnect);
                                break 'outer;
                            }
                        };
                        println!("Line: {:?}", line);
                        if let Some(update) = self.handle_line(line) {
                            updates.push(update);
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => return Err(e)
            }
        }
        Ok(updates)
    }

    /// Checks if the client is listening to the given channel
    pub fn is_listening_to(&self, action: &str) -> bool {
        ::shared::listening_to(&self.listening_to, action)
    }

    /// Update the client with a given event.
    /// 
    /// Specifically,
    /// if the event has the `is_writable` flag set, this will call [Client::process_write].
    /// If the event has the `is_readable` flag set, this will call [Client::process_read].
    pub fn update(&mut self, event: Event) -> Vec<ClientUpdate> {
        if event.readiness().is_writable() {
            if let Err(e) = self.process_write() {
                println!("Client {:?} error:", self.address);
                println!("{:?}", e);
                return vec![
                    ClientUpdate::Disconnect
                ];
            }
        }
        if event.readiness().is_readable() {
            match self.process_read() {
                Ok(v) => v,
                Err(e) => {
                    self.print_error(e);
                    vec![
                        ClientUpdate::Disconnect
                    ]
                }
            }
        } else {
            Vec::new()
        }
    }

    /// Print a debuggable message about this client.
    /// 
    /// TODO: Implement a central logger for this
    pub fn print_error<T: ::std::fmt::Debug>(&self, e: T) {
        println!("Client {:?} error:", self.address);
        println!("{:?}", e);
    }
}

/// Contains the different updates a client can request from a server.
#[derive(Debug)]
pub enum ClientUpdate {
    /// Broadcast a given message to all clients
    Broadcast(Value),

    /// This client has identified with the given name
    Identified(String),

    /// This client has requested a list of all nodes that are connected to the server
    ListNodes,

    /// This client has disconnected
    Disconnect,
}
