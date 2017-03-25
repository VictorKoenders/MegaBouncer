#![deny(warnings)]

extern crate shared;
extern crate base64;
extern crate mio;

pub mod tcp_listener;
pub mod client;

fn main() {
    let mut client = shared::Client::default();
    client.register::<tcp_listener::TcpListener>();
    client.start("TCP Connector");
}
