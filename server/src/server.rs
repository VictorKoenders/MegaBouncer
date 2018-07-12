use client::{Client, ClientUpdate};
use serde_json::{Map, Value};
use shared::mio::net::TcpStream;
use shared::mio::{Event, Token};
use std::collections::HashMap;
use std::net::SocketAddr;
use uuid::Uuid;

/// Temporarily holds a server. This will be consumed with the `start` fn
#[derive(Debug, Default)]
pub struct Server {
    clients: HashMap<Token, Client>,
}

impl Server {
    /// Add a client to the server list
    pub fn add(&mut self, stream: TcpStream, addr: SocketAddr, token: Token) {
        println!("Accepted connection from {:?} ({:?})", addr, token);
        self.clients.insert(token, Client::new(addr, stream));
    }

    /// Handle an incoming event.
    /// 
    /// Specifically, this will look up the associated [Client] object, and call [Server::handle_updates].
    pub fn handle(&mut self, event: Event) {
        let updates;
        let id;
        let name;
        if let Some(client) = self.clients.get_mut(&event.token()) {
            updates = client.update(event);
            name = client.name.clone();
            id = client.id;
        } else {
            return;
        }
        if updates.iter().any(|u| u.is_disconnect()) {
            self.clients.remove(&event.token());
        }
        self.handle_updates(updates, event.token(), name, id);
    }

    /// Handle a list of updates that were received from a client.
    /// 
    /// Optionally the token of the client can be provided.
    fn handle_updates(
        &mut self,
        updates: Vec<ClientUpdate>,
        token: Token,
        name: Option<String>,
        sender_id: Uuid,
    ) {
        for update in updates {
            match update {
                ClientUpdate::SendTo(id, mut message) => {
                    if let Some(name) = &name {
                        message
                            .as_object_mut()
                            .unwrap()
                            .insert(String::from("sender"), Value::String(name.clone()));
                        message
                            .as_object_mut()
                            .unwrap()
                            .insert(String::from("sender_id"), Value::String(sender_id.to_string()));
                    }
                    let mut err = None;
                    {
                        if let Some(client) = self.clients.values_mut().find(|c| c.id.to_string() == id) {
                            if let Err(e) = client.send(&message) {
                                err = Some(e.to_string());
                            }
                        } else {
                            err = Some(format!("Client {:?} not found", id));
                        }
                    }
                    if let Some(err) = err {
                        if let Some(client) = self.clients.get_mut(&token) {
                            client.print_error(err);
                        }
                    }
                }
                ClientUpdate::Broadcast(mut message) => {
                    if let Some(name) = &name {
                        message
                            .as_object_mut()
                            .unwrap()
                            .insert(String::from("sender"), Value::String(name.clone()));
                        message
                            .as_object_mut()
                            .unwrap()
                            .insert(String::from("sender_id"), Value::String(sender_id.to_string()));
                    }
                    self.broadcast(&message);
                }
                ClientUpdate::Identified(name) => {
                    self.broadcast(&make_client_joined(&name, &sender_id));
                }
                ClientUpdate::ListNodes => {
                    let message = self.get_list();
                    let name;
                    let id;
                    if let Some(client) = self.clients.get_mut(&token) {
                        if let Err(e) = client.send(&message) {
                            client.print_error(e);
                            name = client.name.take();
                            id = client.id;
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                    self.clients.remove(&token);
                    if let Some(name) = name {
                        self.broadcast(&make_client_disconnected(&name, &id));
                    }
                }
                ClientUpdate::Disconnect => {
                    if let Some(name) = &name {
                        self.broadcast(&make_client_disconnected(name, &sender_id));
                    }
                }
            }
        }
    }

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
        let mut clients_failed = Vec::new();
        for (token, client) in &mut self.clients {
            if client.is_listening_to(action) {
                if let Err(e) = client.send(message) {
                    client.print_error(e);
                    let val = if let Some(name) = client.name.take() {
                        Some((client.id, name))
                    } else {
                        None
                    };
                    clients_failed.push((token.clone(), val));
                }
            }
        }

        for token in &clients_failed {
            self.clients.remove(&token.0);
        }
        for token in clients_failed {
            if let Some((id, name)) = token.1 {
                self.broadcast(&make_client_disconnected(&name, &id));
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

/// Create an error object with the given name
pub fn make_error(msg: &str) -> Value {
    Value::Object({
        let mut map = Map::new();
        map.insert(String::from("action"), Value::String(String::from("error")));
        map.insert(String::from("message"), Value::String(String::from(msg)));
        map
    })
}

/// Create an "node.identified" object with the given name and id
pub fn make_client_joined(name: &str, uuid: &Uuid) -> Value {
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
pub fn make_client_disconnected(name: &str, uuid: &Uuid) -> Value {
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
pub fn make_client_listening_to(name: &str, channel: &str, uuid: &Uuid) -> Value {
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
