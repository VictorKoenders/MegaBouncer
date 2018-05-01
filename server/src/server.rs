use client::Client;
use futures::sync::mpsc::{channel, Receiver};
use futures::Stream;
use serde_json::{Map, Value};
use serverhandle::ServerHandle;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_io::io::WriteHalf;
use uuid::Uuid;

/// The different messages that each client can emit to the central server
#[derive(Debug)]
pub enum ServerMessage {
    /// Client is connected
    ClientConnected(SocketAddr, WriteHalf<TcpStream>),
    /// Client is disconnected, for whatever reason
    ClientDisconnected(SocketAddr),
    /// Client has send a valid JSON message
    Message(SocketAddr, Value),
}

/// Temporarily holds a server. This will be consumed with the `start` fn
#[derive(Debug)]
pub struct Server {
    receiver: Receiver<ServerMessage>,
}

/// The state of the server
#[derive(Default, Debug)]
struct ServerState {
    clients: HashMap<SocketAddr, Client>,
}

impl ServerState {
    /// Broadcast a given message to every client that is listening to this action
    fn broadcast(&mut self, message: &Value) {
        let action = if let Some(action) = message
            .as_object()
            .and_then(|o| o.get("action").and_then(|s| s.as_str()))
        {
            action
        } else {
            println!("Could not broadcast {:?}", message);
            return;
        };
        for client in self.clients.values_mut() {
            if client.is_listening_to(action) {
                client.send(message);
            }
        }
    }

    /// Get a JSON object that contains info on the currently connected clients
    fn get_list(&self) -> Value {
        Value::Object({
            let mut map = Map::new();
            map.insert(
                String::from("action"),
                Value::String(String::from("node.listed")),
            );
            map.insert(
                String::from("nodes"),
                Value::Array(
                    self.clients
                        .values()
                        .filter_map(|c| {
                            let mut map = Map::new();
                            map.insert(String::from("id"), Value::String(c.id.to_string()));
                            map.insert(String::from("name"), Value::String(c.name.clone()?));
                            map.insert(
                                String::from("channels"),
                                Value::Array(
                                    c.listening_to.iter().cloned().map(Value::String).collect(),
                                ),
                            );
                            Some(Value::Object(map))
                        })
                        .collect(),
                ),
            );
            map
        })
    }
}

/// Handle a node.identify message
/// If the message has a valid "name" property, the name will be set
/// Else the client wil receive an error
fn node_identify(addr: &SocketAddr, state: &mut ServerState, message: &Value) {
    match message
        .as_object()
        .and_then(|o| o.get("name"))
        .and_then(|s| s.as_str())
    {
        Some(s) if !s.trim().is_empty() => {
            let mut uuid = None;
            if let Some(ref mut client) = state.clients.get_mut(&addr) {
                if client.name.is_some() {
                    client.send(&make_error("Already identified"));
                    return;
                }
                client.name = Some(s.to_string());
                uuid = Some(client.id);
            }
            if let Some(uuid) = uuid {
                state.broadcast(&make_client_joined(s, &uuid));
            }
        }
        _ => {
            if let Some(ref mut client) = state.clients.get_mut(&addr) {
                client.send(&make_error("Missing required field 'name'"));
            }
        }
    }
}

/// Register a channel that the current client is listening to
fn node_register(addr: &SocketAddr, state: &mut ServerState, message: &Value) {
    match message
        .as_object()
        .and_then(|o| o.get("channel"))
        .and_then(|s| s.as_str())
    {
        Some(s) if !s.trim().is_empty() => {
            let mut uuid = None;
            let mut name = None;
            if let Some(ref mut client) = state.clients.get_mut(&addr) {
                if client.name.is_none() {
                    client.send(&make_error("Not identified"));
                } else {
                    client.listening_to.push(s.to_string());
                    uuid = Some(client.id);
                    name = client.name.clone();
                }
            }
            if let Some(uuid) = uuid {
                if let Some(name) = name {
                    state.broadcast(&make_client_listening_to(&name, s, &uuid));
                }
            }
        }
        _ => {
            if let Some(ref mut client) = state.clients.get_mut(&addr) {
                client.send(&make_error("Missing required field 'channel'"));
            }
        }
    }
}

/// Send a list to the client with all the other connected nodes
fn node_list(addr: &SocketAddr, state: &mut ServerState) {
    let list = state.get_list();
    if let Some(ref mut client) = state.clients.get_mut(addr) {
        if client.name.is_some() {
            client.send(&list);
        } else {
            client.send(&make_error("Not identified"));
        }
    }
}

/// Send a broadcast message to all nodes in the network that are interested
fn node_broadcast(addr: &SocketAddr, state: &mut ServerState, message: Value) {
    let mut valid = false;
    let mut message = message;
    if let Some(ref mut client) = state.clients.get_mut(addr) {
        if client.name.is_some() {
            if let Some(ref mut o) = message.as_object_mut() {
                o.insert(
                    String::from("name"),
                    Value::String(client.name.clone().unwrap()),
                );
                o.insert(String::from("id"), Value::String(client.id.to_string()));
                valid = true;
            }
        } else {
            client.send(&make_error("Not identified"));
        }
    }
    if valid {
        state.broadcast(&message);
    }
}

impl Server {
    pub fn new() -> (ServerHandle, Server) {
        let (sender, receiver) = channel(10);

        (ServerHandle::new(sender), Server { receiver })
    }

    /// Start the server
    /// This will wait for messages from clients and handle them appropriately
    pub fn start(self) -> ::EmptyFuture {
        use std::sync::Arc;
        use std::sync::Mutex;
        let state = Arc::new(Mutex::new(ServerState::default()));
        Box::new(self.receiver.for_each(move |message| {
            let mut state = state.lock().unwrap();
            match message {
                ServerMessage::ClientConnected(addr, write) => {
                    state.clients.insert(addr, Client::new(addr, write));
                }
                ServerMessage::ClientDisconnected(addr) => {
                    if let Some(mut c) = state.clients.remove(&addr) {
                        if let Some(name) = c.name.take() {
                            state.broadcast(&make_client_disconnected(&name, &c.id));
                        }
                    }
                }
                ServerMessage::Message(addr, message) => match message
                    .as_object()
                    .and_then(|o| o.get("action"))
                    .and_then(|s| s.as_str())
                {
                    Some("node.identify") => node_identify(&addr, &mut state, &message),
                    Some("node.listener.register") => node_register(&addr, &mut state, &message),
                    Some("node.list") => node_list(&addr, &mut state),
                    Some(_) => node_broadcast(&addr, &mut state, message),
                    None => {
                        if let Some(ref mut client) = state.clients.get_mut(&addr) {
                            client.send(&make_error("Missing required field 'action'"));
                        }
                    }
                },
            }
            Ok(())
        }))
    }
}

/// Create an error object with the given name
fn make_error(msg: &str) -> Value {
    Value::Object({
        let mut map = Map::new();
        map.insert(String::from("action"), Value::String(String::from("error")));
        map.insert(String::from("message"), Value::String(String::from(msg)));
        map
    })
}

/// Create an "node.identified" object with the given name and id
fn make_client_joined(name: &str, uuid: &Uuid) -> Value {
    Value::Object({
        let mut map = Map::new();
        map.insert(
            String::from("action"),
            Value::String(String::from("node.identified")),
        );
        map.insert(String::from("name"), Value::String(String::from(name)));
        map.insert(String::from("id"), Value::String(uuid.to_string()));
        map
    })
}

/// Create an "node.disconnected" object with the given name and id
fn make_client_disconnected(name: &str, uuid: &Uuid) -> Value {
    Value::Object({
        let mut map = Map::new();
        map.insert(
            String::from("action"),
            Value::String(String::from("node.disconnected")),
        );
        map.insert(String::from("name"), Value::String(String::from(name)));
        map.insert(String::from("id"), Value::String(uuid.to_string()));
        map
    })
}

/// Create an "node.channel.registered" object with the given name, channel and id
fn make_client_listening_to(name: &str, channel: &str, uuid: &Uuid) -> Value {
    Value::Object({
        let mut map = Map::new();
        map.insert(
            String::from("action"),
            Value::String(String::from("node.channel.registered")),
        );
        map.insert(String::from("name"), Value::String(String::from(name)));
        map.insert(
            String::from("channel"),
            Value::String(String::from(channel)),
        );
        map.insert(String::from("id"), Value::String(uuid.to_string()));
        map
    })
}
