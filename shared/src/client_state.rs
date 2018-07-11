use serde_json::Value;
use client::{ChannelListener, TokenListener, StartupListener};
use mio_poll_wrapper::Handle;
use std::collections::HashMap;
use ChannelUpdate;

pub struct ClientState<TState: Send> {
    pub name: String,
    pub state: TState,
    pub user_interface: Option<String>,
    pub listeners: HashMap<String, ChannelListener<TState>>,
    pub evented_listener: Option<TokenListener<TState>>,
    pub startup_listener: Option<StartupListener<TState>>,
}

impl<TState: Send> ClientState<TState> {
    pub fn new(name: String, state: TState) -> Self {
        ClientState {
            name,
            state,
            user_interface: None,
            listeners: HashMap::new(),
            evented_listener: None,
            startup_listener: None,
        }
    }
    pub fn json_received(&mut self, json: &Value, handle: &mut Handle) {
        let action = match json.as_object()
            .and_then(|o| o.get("action"))
            .and_then(|a| a.as_str())
        {
            Some(a) => a,
            None => {
                println!("JSON does not have an action");
                println!("{:?}", json);
                return;
            }
        };
        let mut update = ChannelUpdate {
            channel: action,
            value: json,
            state: &mut self.state,
            handle,
        };

        for listener in self.listeners.iter().filter_map(|(k, v)| {
            if ::listening_to(&[k], action) {
                Some(v)
            } else {
                None
            }
        }) {
            listener(&mut update);
        }
    }
}
