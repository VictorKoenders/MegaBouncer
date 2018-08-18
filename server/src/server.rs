use client::{Client, ClientUpdate};
use message::{
    add_client_sender_to_message, make_client_disconnected, make_client_joined, make_node_list,
    FIELD_ACTION,
};
use serde_json::Value;
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
        self.handle_updates(updates, event.token(), &name, id);
    }

    /// Handle a list of updates that were received from a client.
    ///
    /// Optionally the token of the client can be provided.
    fn handle_updates(
        &mut self,
        updates: Vec<ClientUpdate>,
        token: Token,
        name: &Option<String>,
        sender_id: Uuid,
    ) {
        for update in updates {
            match update {
                ClientUpdate::SendTo(id, mut message) => {
                    if let Some(name) = &name {
                        add_client_sender_to_message(&mut message, name, &sender_id)
                    }
                    let mut err = None;
                    {
                        if let Some(client) =
                            self.clients.values_mut().find(|c| c.id.to_string() == id)
                        {
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
                        add_client_sender_to_message(&mut message, name, &sender_id);
                    }
                    self.broadcast(&message);
                }
                ClientUpdate::Identified(name) => {
                    self.broadcast(&make_client_joined(&name, &sender_id));
                }
                ClientUpdate::ListNodes => {
                    let message = make_node_list(self.clients.values());
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
            .and_then(|o| o.get(FIELD_ACTION).and_then(|s| s.as_str()))
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
                    clients_failed.push((*token, val));
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
}
