use super::{Channel, Message}; //, Uuid};
use mio::{Event, Poll, Token};
use serde_json::Value;

pub trait Component {
    fn init(&self, poll: &Poll) -> Vec<ComponentResponse>;
    fn message_received(&mut self, poll: &Poll, channel: &Channel, message: &Value) -> Vec<ComponentResponse>;
    // fn reply_received(&mut self, _poll: &Poll, _uuid: Uuid, _channel: &Option<Channel>, _message: &Value) -> Vec<ComponentResponse> {
    //     Vec::new()
    // }
    fn node_connected(&mut self, _poll: &Poll, _name: &str) -> Vec<ComponentResponse>{
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
    //Reply(Value),
    Send(Message),
    RemoveSelf,
}
