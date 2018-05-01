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

#[derive(Debug)]
pub enum ServerMessage {
    ClientConnected(SocketAddr, WriteHalf<TcpStream>),
    ClientDisconnected(SocketAddr),
    Message(SocketAddr, Value),
}

#[derive(Debug)]
pub struct Server {
    receiver: Receiver<ServerMessage>,
}
#[derive(Default, Debug)]
struct ServerState {
    clients: HashMap<SocketAddr, Client>,
}

impl ServerState {
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
        println!("Broadcasting {:?}", message);
        for client in self.clients.values_mut() {
            if client.is_listening_to(action) {
                client.send(message);
            }
        }
    }
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
                            map.insert(
                                String::from("name"),
                                Value::String(c.name.clone()?),
                            );
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

impl Server {
    pub fn new() -> (ServerHandle, Server) {
        let (sender, receiver) = channel(10);

        (ServerHandle::new(sender), Server { receiver })
    }

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
                    Some("node.identify") => match message
                        .as_object()
                        .and_then(|o| o.get("name"))
                        .and_then(|s| s.as_str())
                    {
                        Some(s) if !s.trim().is_empty() => {
                            let mut uuid = None;
                            if let Some(ref mut client) = state.clients.get_mut(&addr) {
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
                    },
                    Some("node.listener.register") => match message
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
                    },
                    Some("node.listener.remove") => {
                        println!("Removing listner {:?}", message);
                    }
                    Some("node.list") => {
                        let list = state.get_list();
                        if let Some(ref mut client) = state.clients.get_mut(&addr) {
                            if client.name.is_some() {
                                client.send(&list);
                            } else {
                                client.send(&make_error("Not identified"));
                            }
                        }
                    }
                    Some(_) => {
                        let mut valid = false;
                        let mut message = message;
                        if let Some(ref mut client) = state.clients.get_mut(&addr) {
                            if client.name.is_some() {
                                if let Some(ref mut o) = message.as_object_mut() {
                                    o.insert(
                                        String::from("name"),
                                        Value::String(client.name.clone().unwrap()),
                                    );
                                    o.insert(
                                        String::from("id"),
                                        Value::String(client.id.to_string()),
                                    );
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

fn make_error(msg: &str) -> Value {
    Value::Object({
        let mut map = Map::new();
        map.insert(String::from("action"), Value::String(String::from("error")));
        map.insert(String::from("message"), Value::String(String::from(msg)));
        map
    })
}

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
