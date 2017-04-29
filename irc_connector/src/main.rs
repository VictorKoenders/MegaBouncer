#![deny(warnings)]

extern crate shared;
extern crate base64;

mod connector;
mod channel;
mod server;
mod message;

pub use channel::IrcChannel;
pub use server::IrcServer;

fn main() {
    let mut client = shared::Client::default();
    client.register::<connector::IrcConnector>();
    client.start("IRC Connector");
}
