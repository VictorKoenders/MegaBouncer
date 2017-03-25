use shared::{ActionType, Channel,  Message};
use std::collections::VecDeque;
use serde_json::{self, Value};
use shared::{Error, Result};
use std::io::{Read, Write};
use std::net::SocketAddr;
use mio::tcp::TcpStream;
use std::convert::From;

pub struct Client {
    socket: TcpStream,
    pub address: SocketAddr,
    name: Option<String>,
    writable: bool,
    readable: bool,
    buffer: Vec<u8>,

    channels: Vec<Channel>,
    write_buffer: VecDeque<Message>,
    write_buffer_bytes: Vec<u8>
}

impl Client {
    pub fn new(socket: TcpStream, address: SocketAddr) -> Client {
        Client {
            socket: socket,
            address: address,
            name: None,
            writable: false,
            readable: false,
            buffer: Vec::new(),
            channels: Vec::new(),
            write_buffer: VecDeque::new(),
            write_buffer_bytes: Vec::new(),
        }
    }

    pub fn set_readable(&mut self, is_readable: bool) {
        self.readable = is_readable;
    }

    pub fn set_writable(&mut self, is_writable: bool) {
        self.writable = is_writable;
        if self.writable {
            self.process_write_queue();
        }
    }

    fn process_write_queue(&mut self) {
        if !self.writable { return; }
        loop {
            if self.write_buffer_bytes.len() > 0 {
                match self.socket.write(&self.write_buffer_bytes) {
                    Ok(length) => {
                        self.write_buffer_bytes.drain(..length);
                        if self.write_buffer_bytes.len() > 0 {
                            println!("Could not write full queue, retrying again next time");
                            self.writable = false;
                            return;
                        }
                    },
                    Err(e) => {
                        panic!("Could not write to stream: {:?}", e);
                    }
                }
            } else if let Some(message) = self.write_buffer.pop_front() {
                self.write_buffer_bytes.extend(message.to_bytes().unwrap().into_iter());
                self.write_buffer_bytes.extend(b"\n");
            } else {
                break;
            }
        }

    }

    pub fn is_listening_to(&self, channel: &Channel) -> bool {
        self.channels.iter().any(|c| c.matches(channel))
    }

    pub fn write(&mut self, message: Message) {
        self.write_buffer.push_back(message);
        if self.writable {
            self.process_write_queue();
        }
    }

    pub fn emit_simple_error<T: ToString>(&mut self, str: T) {
        self.try_send(Message::from_error_string(str));
    }

    fn try_identify(&mut self, message: Message, result: &mut Vec<ClientEvent>) -> Result<()> {
        if self.name.is_some() {
            self.emit_simple_error("Name is already set");
        } else if let Value::Object(ref map) = message.data {
            if let Some(&Value::String(ref str)) = map.get("name") {
                self.name = Some(str.clone());
                result.push(ClientEvent::Broadcast(message.clone()));
            }
        }
        Ok(())
    }

    fn has_name(&mut self) -> bool {
        if self.name.is_some() {
            true
        } else {
            self.write_buffer.push_back(Message::from_error_string("Name not set, please identify first"));
            false
        }
    }

    fn try_register_listener(&mut self,
                             message: Message,
                             result: &mut Vec<ClientEvent>)
                             -> Result<()> {
        if !self.has_name() {
            return Ok(())
        }
        let channel = match message.channel {
            None => {
                self.write_buffer.push_back(Message::from_error_string("Channel required with RegisterListener action"));
                return Ok(());
            },
            Some(c) => Channel::from_string(c)
        };
        let channel_string = channel.to_string();
        self.channels.push(channel);
        let message = Message::new_emit("client.listener.register", |map| {
            map.insert(String::from("channel"), Value::String(channel_string));
            map.insert(String::from("client"), Value::String(self.name.clone().unwrap()));
        });
        result.push(ClientEvent::Broadcast(message));
        Ok(())
    }

    fn try_forget_listener(&mut self,
                           message: Message,
                           result: &mut Vec<ClientEvent>)
                           -> Result<()> {
        if !self.has_name() {
            return Ok(())
        }
        let channel: Channel = match message.channel {
            None => {
                self.write_buffer.push_back(Message::from_error_string("Channel required with ForgetListener action"));
                return Ok(());
            },
            Some(c) => c
        };
        while let Some(index) = self.channels.iter().position(|c| channel.matches(c)) {
            let old_channel = self.channels.remove(index);
            let message = Message::new_emit("client.listener.forget", |map| {
                map.insert(String::from("channel"), Value::String(old_channel.to_string()));
                map.insert(String::from("client"), Value::String(self.name.clone().unwrap()));
            });
            result.push(ClientEvent::Broadcast(message));
        }
        Ok(())
    }

    fn try_get_listeners(&mut self, _message: Message, _result: &mut Vec<ClientEvent>) -> Result<()> {
        println!("Not implemented: try_get_listeners");
        Ok(())
    }

    fn try_get_clients(&mut self, _message: Message, _result: &mut Vec<ClientEvent>) -> Result<()> {
        println!("Not implemented: try_get_clients");
        Ok(())
    }

    fn try_get_response(&mut self, _message: Message, _result: &mut Vec<ClientEvent>) -> Result<()> {
        println!("Not implemented: try_get_response");
        Ok(())
    }

    fn try_log_error(&mut self, _message: Message, _result: &mut Vec<ClientEvent>) -> Result<()> {
        println!("Not implemented: try_log_error");
        Ok(())
    }

    fn try_emit(&mut self, message: Message, result: &mut Vec<ClientEvent>) -> Result<()> {
        // TODO: implement a reply token
        result.push(ClientEvent::Broadcast(message));
        Ok(())
    }

    fn handle_line(&mut self, line: String) -> Result<Vec<ClientEvent>> {
        let value: Value = serde_json::from_str(line.trim())?;
        let message = Message::from_json(value)?;
        let mut result = Vec::new();
        match message.action {
            ActionType::Identify => self.try_identify(message, &mut result)?,
            ActionType::RegisterListener => self.try_register_listener(message, &mut result)?,
            ActionType::ForgetListener => self.try_forget_listener(message, &mut result)?,
            ActionType::GetListeners => self.try_get_listeners(message, &mut result)?,
            ActionType::GetClients => self.try_get_clients(message, &mut result)?,
            ActionType::Response => self.try_get_response(message, &mut result)?,
            ActionType::Error => self.try_log_error(message, &mut result)?,
            ActionType::Emit => self.try_emit(message, &mut result)?,
        };
        Ok(result)
    }

    pub fn try_send(&mut self, message: Message) {
        // TODO: Check if the client accepts this message type
        self.write(message);
    }

    pub fn read_data(&mut self) -> Result<Vec<ClientEvent>> {
        if !self.readable {
            return Err(Error::new_client_not_readable(self.address.to_string()));
        }
        let mut has_read_data = false;
        let mut events = Vec::new();
        const BUFFER_SIZE: usize = 256;
        const MAX_BUFFER_LENGTH: usize = 256*1024;

        loop {
            let mut buffer = [0u8; BUFFER_SIZE];
            let length = match self.socket.read(&mut buffer) {
                Ok(length) => length,
                Err(e) => {
                    return if has_read_data {
                        self.readable = false;
                        Ok(events)
                    } else {
                        Err(From::from(e))
                    }
                }
            };

            if length == 0 {
                if !has_read_data {
                    events.push(ClientEvent::Disconnect);
                }
                return Ok(events);
            }
            has_read_data = true;
            self.buffer.extend(&buffer[0..length]);

            if self.buffer.len() > MAX_BUFFER_LENGTH {
                println!("{} Client {:?} killed because of the buffer being too big ({} / {})",
                         super::server::get_time(),
                         self.address,
                         self.buffer.len(),
                         MAX_BUFFER_LENGTH);
                self.buffer.clear();
                self.write(Message::from_error_string("Buffer out of range"));
                events.push(ClientEvent::Disconnect);
                return Ok(events);
            }

            while let Some(index) = self.buffer.iter().position(|c| *c == b'\n') {
                let line = self.buffer.drain(0..index + 1).collect();
                if let Ok(line) = String::from_utf8(line) {
                    match self.handle_line(line) {
                        Ok(e) => events.extend(e.into_iter()),
                        Err(e) => self.write(Message::from_error(e)),
                    };
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClientEvent {
    Disconnect,
    Broadcast(Message),
}
