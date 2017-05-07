#![deny(warnings)]
#![feature(try_from)]

#[macro_use] extern crate lazy_static;
extern crate serde_json;
extern crate uuid;
extern crate mio;

mod component_wrapper;
pub mod writeable;
mod message_reply;
mod component;
mod message;
mod channel;
mod client;
mod error;

pub use component::{Component, ComponentResponse};
pub use component_wrapper::ComponentWrapper;
pub use message_reply::MessageReply;
pub use error::{Error, Result};
pub use serde_json::Value;
pub use message::Message;
pub use channel::Channel;
pub use client::Client;
pub use uuid::Uuid;

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
    pub use ::channel::{
        REGISTER_LISTENER,
        FORGET_LISTENER,
        LIST_CLIENTS,
        IDENTIFY,
        REPLY,
        ERROR
    };
    pub use ::uuid::Uuid;
}
