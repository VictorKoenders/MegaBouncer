#![deny(warnings)]
#![feature(try_from)]

extern crate serde_json;
extern crate mio;

mod component_wrapper;
pub mod writeable;
mod component;
mod message;
mod channel;
mod client;
mod error;

pub use component::{Component, ComponentResponse};
pub use component_wrapper::ComponentWrapper;
pub use error::{Error, Result};
pub use message::Message;
pub use channel::Channel;
pub use client::Client;

pub const ACTION_NAME: &'static str = "action";

pub mod prelude {
    pub use super::{
        Component,
        ComponentResponse,
        Result,
        Message,
        Channel,
        Client,
    };
    pub use ::serde_json::{
        Map,
        Value
    };
    pub use ::mio::{
        Event,
        Poll
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionType {
    RegisterListener,
    ForgetListener,
    GetListeners,
    GetClients,
    Identify,
    Response,
    Error,
    Emit,
}

impl ActionType {
    pub fn from_str(str: &str) -> Option<ActionType> {
        let types = ActionType::default_types();
        Some(match str {
            x if x == types[0] => ActionType::RegisterListener,
            x if x == types[1] => ActionType::ForgetListener,
            x if x == types[2] => ActionType::GetListeners,
            x if x == types[3] => ActionType::GetClients,
            x if x == types[4] => ActionType::Identify,
            x if x == types[5] => ActionType::Response,
            x if x == types[6] => ActionType::Error,
            x if x == types[7] => ActionType::Emit,
            _ => return None
        })
    }
    pub fn default_types() -> [&'static str;8] {
        [
            "register_listener",
            "forget_listener",
            "get_listeners",
            "get_clients",
            "identify",
            "response",
            "error",
            "emit",
        ]
    }
}

impl std::string::ToString for ActionType {
    fn to_string(&self) -> String {
        let types = ActionType::default_types();
        match *self {
            ActionType::RegisterListener => types[0].to_string(),
            ActionType::ForgetListener => types[1].to_string(),
            ActionType::GetListeners => types[2].to_string(),
            ActionType::GetClients => types[3].to_string(),
            ActionType::Identify => types[4].to_string(),
            ActionType::Response => types[5].to_string(),
            ActionType::Error => types[6].to_string(),
            ActionType::Emit => types[7].to_string(),
        }
    }
}
