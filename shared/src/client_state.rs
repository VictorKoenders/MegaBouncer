use client::{ChannelListener, StartupListener, TokenListener};
use mio::Token;
use mio_extras::channel::Receiver;
use mio_poll_wrapper::Handle;
use serde_json::{Value, Map};
use std::collections::HashMap;
use ChannelUpdate;

pub enum ClientMessage {
    Send(Value),
}

pub struct ClientState<TState> {
    pub name: String,
    pub state: TState,
    pub user_interface: Option<String>,
    pub receiver: Option<Receiver<ClientMessage>>,
    pub receiver_token: Token,
    pub listeners: HashMap<String, ChannelListener<TState>>,
    pub evented_listener: Option<TokenListener<TState>>,
    pub startup_listener: Option<StartupListener<TState>>,
}

#[derive(Default)]
pub struct EmitOrReply {
    pub emit: Vec<Value>,
    pub reply: Vec<(String, Value)>,
}

impl EmitOrReply {
    pub fn extend(&mut self, other: EmitOrReply) {
        self.emit.extend(other.emit.into_iter());
        self.reply.extend(other.reply.into_iter());
    }
}

impl<TState: Send> ClientState<TState> {
    pub fn new(name: String, state: TState) -> Self {
        ClientState {
            name,
            state,
            user_interface: None,
            receiver: None,
            receiver_token: Token(0),
            listeners: HashMap::new(),
            evented_listener: None,
            startup_listener: None,
        }
    }
    pub fn json_received(&mut self, json: &Value, handle: &mut Handle) -> EmitOrReply {
        let action = match json
            .as_object()
            .and_then(|o| o.get("action"))
            .and_then(|a| a.as_str())
        {
            Some(a) => a,
            None => {
                println!("JSON does not have an action");
                println!("{:?}", json);
                return EmitOrReply::default();
            }
        };
        let sender_id = json
            .get("sender_id")
            .and_then(|r| r.as_str())
            .map(|s| s.to_string());
        let mut update = ChannelUpdate {
            channel: action,
            value: json,
            emit: Vec::new(),
            reply: Vec::new(),
            state: &mut self.state,
            handle,
        };

        if action == "ui.get" {
            if let Some(ref ui) = self.user_interface {
                update.reply.push(Value::Object({
                    let mut map = Map::new();
                    map.insert("action".to_string(), Value::String("ui.gotten".to_string()));
                    map.insert("ui".to_string(), Value::String(ui.clone()));
                    map
                }));
            }
        }

        for listener in self.listeners.iter().filter_map(|(k, v)| {
            if ::listening_to(&[k], action) {
                Some(v)
            } else {
                None
            }
        }) {
            listener(&mut update);
        }
        EmitOrReply {
            emit: update.emit,
            reply: if let Some(sender_id) = sender_id {
                update
                .reply
                .into_iter()
                .map(|r| (sender_id.clone(), r))
                .collect()
            } else {
                Vec::new()
            },
        }
    }
}
