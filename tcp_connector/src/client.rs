use std::io::{Read, Write};
use std::string::ToString;
use std::net::SocketAddr;
use mio::tcp::TcpStream;
use shared::prelude::*;
use mio::Token;

pub struct TcpClient {
    pub host: String,
    pub port: u16,
    pub stream: TcpStream,
    pub addr: SocketAddr,
    pub token: Token,
    pub status: TcpClientStatus,
    
    pub writable: bool,
    pub outgoing_buffer: Vec<u8>,
}

impl TcpClient {
    pub fn new(host: String, port: u16, stream: TcpStream, addr: SocketAddr, token: Token) -> TcpClient {
        TcpClient {
            host: host,
            port: port,
            stream: stream,
            addr: addr,
            token: token,
            status: TcpClientStatus::Connected,
            writable: false,
            outgoing_buffer: Vec::new(),
        }
    }

    pub fn try_write(&mut self, force_writable: bool) {
        if force_writable {
            self.writable = true;
        } else if !self.writable {
            return;
        }

        match self.stream.write(&self.outgoing_buffer) {
            Ok(length) => {
                if length < self.outgoing_buffer.len() {
                    self.writable = false;
                }
                self.outgoing_buffer.drain(..length);
            },
            Err(e) => {
                self.status = TcpClientStatus::Disconnected;
                println!("Could not write to stream: {:?}", e);
            }
        };
    }

    fn try_read(&mut self, response: &mut Vec<ComponentResponse>){
        let mut buffer_vec = Vec::new();
        let mut buffer = [0u8;256];
        loop {
            match self.stream.read(&mut buffer) {
                Ok(length) => {
                    buffer_vec.extend(&buffer[0..length]);
                },
                Err(e) => {
                    if !buffer_vec.is_empty() {
                        let host = self.host.clone();
                        response.push(ComponentResponse::Send(Message::new_emit("tcp.data", |map| {
                            map.insert(String::from("host"), Value::String(host));
                            map.insert(String::from("port"), Value::Number(self.port.into()));
                            map.insert(String::from("data"), Value::String(::base64::encode(&buffer_vec)));
                        })));
                    } else {
                        println!("Could not read any data: {:?}", e);
                        self.status = TcpClientStatus::Disconnected;
                        response.push(self.get_status_message());
                    }
                    return;
                }
            }
        }
    }

    fn get_status_message(&self) -> ComponentResponse {
        ComponentResponse::Send(Message::new_emit("tcp.status", |map| {
            map.insert(String::from("host"), Value::String(self.host.clone()));
            map.insert(String::from("port"), Value::Number(self.port.into()));
            map.insert(String::from("status"), Value::String(self.status.to_string()));
        }))
    }

    pub fn handle(&mut self, event: &Event) -> Vec<ComponentResponse> {
        let readiness = event.readiness();
        let mut response = Vec::new();
        if readiness.is_writable() {
            self.try_write(true);
            if self.status == TcpClientStatus::Disconnected {
                response.push(self.get_status_message());
            }
        }
        if readiness.is_readable() {
            self.try_read(&mut response);
        }
        response
    }
}

#[derive(PartialEq)]
pub enum TcpClientStatus {
    Connected,
    Disconnected
}

impl ToString for TcpClientStatus {
    fn to_string(&self) -> String {
        match *self {
            TcpClientStatus::Connected => String::from("Connected"),
            TcpClientStatus::Disconnected => String::from("Disconnected"),
        }
    }
}
