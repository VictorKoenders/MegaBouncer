#![allow(dead_code)]

extern crate base64;
extern crate shared;

use shared::mio::net::TcpStream;
use shared::mio::Token;
use shared::mio_poll_wrapper::Handle;
use shared::serde_json::{Value, Map};
use shared::ChannelUpdate;
use std::io::{ErrorKind, Write};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

fn main() {
    let mut client = shared::client::Client::new("Tcp connector", State::default());
    client.register_listener("tcp.listen", listen);
    client.register_listener("tcp.write", write);
    client.register_listener("tcp.status", status);
    client.register_listener("get_ui", get_ui);
    client.register_listener("*", print_all);
    client.launch();
}
fn print_all(update: &mut ChannelUpdate<State>){
    println!("{:?}", update.value);
}

fn get_ui(update: &mut ChannelUpdate<State>){
    println!("Getting UI");
    update.reply.push(Value::Object({
        let mut map = Map::new();
        map.insert("action".to_string(), Value::String("ui_gotten".to_string()));
        map.insert("ui".to_string(), Value::String("<b>Hello from tcp connector</b>".to_string()));
        map
    }));
}

fn listen(update: &mut ChannelUpdate<State>) {
    let host = match update.value.get("host") {
        Some(Value::String(s)) => s,
        _ => return,
    };
    let port = match update.value.get("port") {
        Some(Value::Number(p)) => p.as_u64().unwrap_or_else(|| 0) as u16,
        _ => return,
    };
    if let Some(_) = update
        .state
        .connections
        .iter()
        .find(|c| &c.host == host && c.port == port)
    {
        status(update);
        return;
    }
    let connection = Connection::new(update.handle, host.clone(), port);
    update.state.connections.push(connection);
    status(update);
}

fn write(update: &mut ChannelUpdate<State>) {
    let host = match update.value.get("host") {
        Some(Value::String(s)) => s,
        _ => return,
    };
    let port = match update.value.get("port") {
        Some(Value::Number(p)) => p.as_u64().unwrap_or_else(|| 0) as u16,
        _ => return,
    };
    let data = match update.value.get("data") {
        Some(Value::String(s)) => match base64::decode(&s) {
            Ok(d) => d,
            Err(e) => {
                println!("Could not decode base64 {:?}", e);
                return;
            }
        },
        _ => return,
    };
    let connection = update.state.get_or_create(update.handle, host, port);
    connection.write(&data);
}

fn status(update: &mut ChannelUpdate<State>) {
    let host = match update.value.get("host") {
        Some(Value::String(s)) => s,
        _ => return,
    };
    let port = match update.value.get("port") {
        Some(Value::Number(p)) => p.as_u64().unwrap_or_else(|| 0) as u16,
        _ => return,
    };
    let _connection = update.state.get_or_create(update.handle, host, port);
    // TODO: Send response
}

#[derive(Default)]
pub struct State {
    pub connections: Vec<Connection>,
}

impl State {
    pub fn get_or_create(&mut self, handle: &mut Handle, host: &str, port: u16) -> &mut Connection {
        let index = if let Some(index) = self.connections.iter().position(|c| &c.host == host && c.port == port) {
            index
        } else {
            self.connections.push(Connection::new(handle, host.to_string(), port));
            self.connections.len() - 1
        };
        &mut self.connections[index]
    }
}

pub struct Connection {
    pub token: Token,
    pub host: String,
    pub port: u16,
    pub stream: TcpStream,
    write_buffer: Vec<u8>,
    is_writable: bool,
    is_readable: bool,
}

impl Connection {
    pub fn new(handle: &mut Handle, host: String, port: u16) -> Connection {
        let addr: IpAddr = IpAddr::from_str(&host).unwrap();
        let addr: SocketAddr = (addr, port).into();
        let mut connection = Connection {
            host,
            port,
            token: Token(0),
            stream: TcpStream::connect(&addr).unwrap(),
            write_buffer: Vec::new(),
            is_writable: false,
            is_readable: false,
        };
        connection.token = handle.register(&connection.stream).unwrap();
        connection
    }
    pub fn write(&mut self, data: &[u8]) {
        self.write_buffer.extend(data);
        if self.is_writable {
            self.process_write_queue();
        }
    }

    fn process_write_queue(&mut self) {
        if self.write_buffer.is_empty() {
            return;
        }
        let mut did_write = false;
        loop {
            match self.stream.write(&self.write_buffer) {
                Ok(0) if did_write => {
                    self.is_writable = self.write_buffer.is_empty();
                    return;
                }
                Ok(0) => return,
                Ok(n) => {
                    self.write_buffer.drain(..n);
                    did_write = true;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => return,
                Err(e) => {
                    println!("{:?}", e);
                    return;
                }
            }
        }
    }
}
