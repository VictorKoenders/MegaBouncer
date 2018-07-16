extern crate shared;
extern crate chrono;

use shared::mio::net::TcpStream;
use shared::mio::Token;
use shared::{ChannelUpdate, Startup};
use chrono::NaiveDateTime;

fn main() {
    let state = State::default();
    let mut client = shared::client::Client::new("Irc connector", state);

    client.on_startup(startup);
    client.register_user_interface("ui/dist/bundle.js");
    client.register_listener("irc.connect", irc_connect);
    client.register_listener("tcp.received", tcp_received);
    client.register_listener("tcp.status", tcp_status);
    client.launch();
}

fn irc_connect(_update: &mut ChannelUpdate<State>) {

}

fn startup(_data: &mut Startup<State>) {

}

fn tcp_received(_update: &mut ChannelUpdate<State>) {

}

fn tcp_status(_update: &mut ChannelUpdate<State>) {

}

#[derive(Default)]
pub struct State {
    pub connections: Vec<Connection>,
}

pub struct Connection {
    pub server_name: String,
    pub server_host: String,
    pub nick: String,
    pub use_ssl: bool,
    pub channels: Vec<Channel>,
    pub stream: Option<Stream>,
}

pub struct Channel {
    pub name: String,
    pub connected: bool,
    pub log: Vec<Message>,
}

pub struct Message {
    pub from: String,
    pub text: String,
    pub timestamp: NaiveDateTime,
}

pub struct Stream {
    pub token: Token,
    pub stream: TcpStream,
    pub buffer: Vec<u8>,
}