use mio::{Token, Ready, PollOpt};
use super::client::TcpClient;
use std::net::ToSocketAddrs;
use std::string::ToString;
use mio::tcp::TcpStream;
use shared::prelude::*;

pub struct TcpListener {
    next_token: Token,
    connections: Vec<TcpClient>,
}

impl Default for TcpListener {
    fn default() -> TcpListener {
        TcpListener {
            next_token: Token(1),
            connections: Vec::new()
        }
    }
}

impl TcpListener {
    fn err<T: ToString>(err: T) -> Vec<ComponentResponse> {
        vec![ComponentResponse::Send(Message::from_error_string(err))]
    }
    fn connect(&mut self, poll: &Poll, message: &Value) -> Vec<ComponentResponse> {
        let map = match message.as_object() {
            Some(map) => map,
            None => return TcpListener::err("Data is not an object")
        };

        let host: String = match map.get("host") {
            Some(&Value::String(ref host)) => host.clone(),
            _ => return TcpListener::err("Missing required field \"host\" (string)")
        };

        let port: i64 = match map.get("port") {
            Some(&Value::Number(ref port)) => {
                match port.as_i64() {
                    Some(p) => p,
                    None => return TcpListener::err(format!("Could not parse port \"{:?}\"", port))
                }
            },
            Some(&Value::String(ref port)) => {
                match port.parse() {
                    Ok(p) => p,
                    Err(e) => return TcpListener::err(format!("Could not parse port {:?}: {:?}", port, e))
                }
            },
            _ => return TcpListener::err("Missing required field \"port\" (number)")
        };

        let addr = format!("{}:{}", host, port);
        
        let addr = match addr.to_socket_addrs().map(|mut a| a.next()) {
            Ok(Some(addr)) => addr,
            _ => return TcpListener::err(format!("Could not resolve host {:?}", addr))
        };

        let token = Token(match self.next_token.0 { 0 => 1, x => x });
        self.next_token = Token(token.0.wrapping_add(1));

        let stream = match TcpStream::connect(&addr) {
            Ok(stream) => stream,
            Err(e) => return TcpListener::err(format!("Could not connect to \"{}:{}\": {:?}", host, port, e))
        };
        poll.register(&stream, token, Ready::readable() | Ready::writable(), PollOpt::edge()).unwrap();
        self.connections.push(TcpClient::new(host.clone(), port as u16, stream, addr, token));
        let last = self.connections.last();
        let ref connection = last.as_ref().unwrap();

        let mut result = Vec::new();
        result.push(ComponentResponse::RegisterToken(connection.token));
        result.push(ComponentResponse::Send(Message::new_emit("tcp.connected", |map|{
            map.insert(String::from("status"), Value::String(connection.status.to_string()));
            map.insert(String::from("host"), Value::String(host));
            map.insert(String::from("port"), Value::Number(port.into()));
        })));
        result
    }
    fn send(&mut self, message: &Value) -> Vec<ComponentResponse> {
        let host = match message.get("host") { Some(&Value::String(ref str)) => str, _ => return TcpListener::err("Could not read host")};
        let port = match message.get("port") {
            Some(&Value::Number(ref nr)) => match nr.as_u64() {
                Some(nr) => nr as u16,
                None => return TcpListener::err("port is not a valid number")
            },
            _ => return TcpListener::err("Could not read port")
        };
        let data = match message.get("data") {
            Some(&Value::String(ref str)) => match ::base64::decode(str) {
                Ok(data) => data,
                Err(e) => return TcpListener::err(format!("Could not decode bas64: {:?}", e))
            },
            _ => return TcpListener::err("Could not read data") 
        };

        match self.connections.iter_mut().find(|c| *c.host == *host && c.port == port) {
            Some(ref mut connection) => {
                connection.outgoing_buffer.extend(&data);
                connection.try_write(false);
                Vec::new()
            },
            None => {
                TcpListener::err(format!("Could not find connection with address {}:{}", host, port))
            }
        }
    }
    fn get_status(&mut self, _message: &Value) -> Vec<ComponentResponse> {
        Vec::new()
    }
    fn disconnect(&mut self, _poll: &Poll, _message: &Value) -> Vec<ComponentResponse> {
        Vec::new()
    }
}

impl Component for TcpListener {
    fn init(&self, _poll: &Poll) -> Vec<ComponentResponse> {
        vec![
            ComponentResponse::ListenToChannel(Channel::from_string("tcp.connect")),
            ComponentResponse::ListenToChannel(Channel::from_string("tcp.send")),
            ComponentResponse::ListenToChannel(Channel::from_string("tcp.status")),
            ComponentResponse::ListenToChannel(Channel::from_string("tcp.disconnect"))
        ]
    }
    fn message_received(&mut self, poll: &Poll, channel: &Channel, message: &Value) -> Vec<ComponentResponse> {
        if channel == "tcp.connect" {
            self.connect(poll, message)
        } else if channel == "tcp.send" {
            self.send(message)
        } else if channel == "tcp.status" {
            self.get_status(message)
        } else if channel == "tcp.disconnect" {
            self.disconnect(poll, message)
        } else {
            vec![
                ComponentResponse::Send(
                    Message::from_error_string(format!("Unknown message \"{:?}\"", channel))
                )
            ]
        }
    }
    fn token_received(&mut self, _poll: &Poll, event: &Event) -> Vec<ComponentResponse>{
        if let Some(ref mut connection) = self.connections.iter_mut().find(|c| c.token == event.token()) {
            connection.handle(event)
        } else {
            Vec::new()
        }
    }
}
