use shared::writeable::{Writeable, WriteQueue};
use shared::{Channel, Message}; //, MessageReply};
use std::collections::VecDeque;
use serde_json::{self, Value};
use shared::{Error, Result};//, Uuid};
use std::net::SocketAddr;
use mio::tcp::TcpStream;
use shared::prelude::*;
use std::convert::From;
use std::io::Read;

//const REPLY_MESSAGE_DEQUE_SIZE: usize = 5;

pub struct Client {
    socket: TcpStream,
    pub address: SocketAddr,
    pub name: Option<String>,
    writeable: bool,
    readable: bool,
    buffer: Vec<u8>,

    channels: Vec<Channel>,
    write_buffer: VecDeque<Message>,
    write_buffer_bytes: Vec<u8>,
    //reply_message_uuids: VecDeque<Uuid>,
}

impl Client {
    pub fn new(socket: TcpStream, address: SocketAddr) -> Client {
        Client {
            socket: socket,
            address: address,
            name: None,
            writeable: false,
            readable: false,
            buffer: Vec::new(),
            channels: Vec::new(),
            write_buffer: VecDeque::new(),
            write_buffer_bytes: Vec::new(),
            //reply_message_uuids: VecDeque::with_capacity(REPLY_MESSAGE_DEQUE_SIZE),
        }
    }

    // pub fn try_accept_reply(&mut self, uuid: &Uuid, message: &Message) -> bool {
    //     if let Some(index) = self.reply_message_uuids.iter().position(|u| u == uuid) {
    //         println!("Sending reply to {:?} ({:?})", self.name, message);
    //         self.try_send(message.clone());
    //         self.reply_message_uuids.remove(index);
    //         true
    //     } else {
    //         false
    //     }
    // }

    pub fn set_readable(&mut self, is_readable: bool) {
        self.readable = is_readable;
    }

    pub fn set_writable(&mut self, is_writable: bool) {
        self.writeable = is_writable;
        self.try_write();
    }

    pub fn is_listening_to(&self, channel: &Channel) -> bool {
        self.channels.iter().any(|c| c.matches(channel))
    }

    pub fn write(&mut self, message: Message) {
        self.write_buffer.push_back(message);
        self.try_write();
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
                result.push(ClientEvent::Broadcast(Message::new_connected_client(str.clone())));
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
            return Ok(());
        }
        let channel = match message.data.as_object().map(|o| o.get("channel")) {
            Some(Some(&Value::String(ref c))) => Channel::from_string(c),
            _ => {
                self.write_buffer.push_back(Message::from_error_string("Channel required with RegisterListener action"));
                return Ok(());
            }
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
            return Ok(());
        }
        let channel = match message.channel {
            None => {
                self.write_buffer.push_back(Message::from_error_string("Channel required with ForgetListener action"));
                return Ok(());
            }
            Some(c) => c,
        };
        while let Some(index) = self.channels.iter().position(|c| channel.matches(c)) {
            let old_channel = self.channels.remove(index);
            let message = Message::new_emit("client.listener.forget", |map| {
                map.insert(String::from("channel"),
                           Value::String(old_channel.to_string()));
                map.insert(String::from("client"),
                           Value::String(self.name.clone().unwrap()));
            });
            result.push(ClientEvent::Broadcast(message));
        }
        Ok(())
    }

    /*
    fn try_get_listeners(&mut self,
                         _message: Message,
                         _result: &mut Vec<ClientEvent>)
                         -> Result<()> {
        println!("Not implemented: try_get_listeners");
        Ok(())
    }
    */

    fn try_get_clients(&mut self, _message: Message, result: &mut Vec<ClientEvent>) -> Result<()> {
        result.push(ClientEvent::SendClients);
        Ok(())
    }

    /*
    fn try_get_response(&mut self,
                        _message: Message,
                        _result: &mut Vec<ClientEvent>)
                        -> Result<()> {
        println!("Not implemented: try_get_response");
        Ok(())
    }

    fn try_log_error(&mut self, _message: Message, _result: &mut Vec<ClientEvent>) -> Result<()> {
        println!("Not implemented: try_log_error");
        Ok(())
    }
    */

    fn try_emit(&mut self, message: Message, result: &mut Vec<ClientEvent>) -> Result<()> {
        // if let MessageReply::ID(uuid) = message.id {
        //     self.reply_message_uuids.push_back(uuid);
        //     if self.reply_message_uuids.len() > REPLY_MESSAGE_DEQUE_SIZE {
        //         self.reply_message_uuids.pop_front();
        //     }
        // }
        result.push(ClientEvent::Broadcast(message));
        Ok(())
    }

    fn handle_line(&mut self, line: String) -> Result<Vec<ClientEvent>> {
        let value: Value = serde_json::from_str(line.trim())?;
        let message = Message::from_json(value)?;
        let mut result = Vec::new();
        if IDENTIFY.is(&message.channel){
            self.try_identify(message, &mut result)?;
        } else if REGISTER_LISTENER.is(&message.channel) {
            self.try_register_listener(message, &mut result)?;
        } else if FORGET_LISTENER.is(&message.channel) {
            self.try_forget_listener(message, &mut result)?;
        } else if LIST_CLIENTS.is(&message.channel) {
            self.try_get_clients(message, &mut result)?;
        } /*else if ERROR.is(message.channel) {
            self.try_log_error(message, &mut result)?;
        }*/ else {
            self.try_emit(message, &mut result)?;
        }

        /*match message.action {
            ActionType::Identify => self.try_identify(message, &mut result)?,
            ActionType::RegisterListener => self.try_register_listener(message, &mut result)?,
            ActionType::ForgetListener => self.try_forget_listener(message, &mut result)?,
            ActionType::GetListeners => self.try_get_listeners(message, &mut result)?,
            ActionType::GetClients => self.try_get_clients(message, &mut result)?,
            ActionType::Response => self.try_get_response(message, &mut result)?,
            ActionType::Error => self.try_log_error(message, &mut result)?,
            ActionType::Emit => self.try_emit(message, &mut result)?,
        };*/
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
        const MAX_BUFFER_LENGTH: usize = 256 * 1024;

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
                    match self.handle_line(line.clone()) {
                        Ok(e) => events.extend(e.into_iter()),
                        Err(e) => {
                            println!("Could not parse line {:?}: {:?}", line, e);
                            self.write(Message::from_error(e));
                        }
                    };
                }
            }
        }
    }
}

impl Writeable<Message> for Client {
    fn get_write_queue(&mut self) -> WriteQueue<Message> {
        WriteQueue {
            byte_queue: &mut self.write_buffer_bytes,
            message_queue: &mut self.write_buffer,
            stream: &mut self.socket,
            writeable: &mut self.writeable,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClientEvent {
    Disconnect,
    Broadcast(Message),
    SendClients,
}
