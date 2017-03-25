#![deny(warnings)]

extern crate shared;
use shared::prelude::*;

fn main() {
    let mut client = shared::Client::default();
    client.register::<LoggingComponent>();
    client.start("Test client");
}

#[derive(Default)]
struct LoggingComponent;

impl shared::Component for LoggingComponent {
    fn init(&self, _poll: &Poll) -> Vec<ComponentResponse> {
        vec![
            ComponentResponse::ListenToChannel(Channel::from_string("*"))
        ]
    }

    fn message_received(&mut self, _poll: &Poll, channel: &Channel, message: &Value) -> Vec<ComponentResponse> {
        println!("Received {:?}: {:?}", channel, message);
        Vec::new()
    }
    fn node_connected(&mut self, _poll: &Poll, name: &String) -> Vec<ComponentResponse>{
        println!("Node {:?} connected", name);
        Vec::new()
    }
}
