use super::{Channel, Message};
use mio::{Event, Poll, Token};
use serde_json::Value;

pub trait Component {
    fn init(&self, poll: &Poll) -> Vec<ComponentResponse>;
    fn message_received(&mut self, poll: &Poll, channel: &Channel, message: &Value) -> Vec<ComponentResponse>;
    fn node_connected(&mut self, _poll: &Poll, _name: &String) -> Vec<ComponentResponse>{
        Vec::new()
    }
    fn token_received(&mut self, _poll: &Poll, _event: &Event) -> Vec<ComponentResponse>{
        Vec::new()
    }
}

pub enum ComponentResponse {
    StopListeningToChannel(Channel),
    SpawnComponent(Box<Component>),
    ListenToChannel(Channel),
    RegisterToken(Token),
    RemoveToken(Token),
    Reply(Value),
    Send(Message),
    RemoveSelf,
}
