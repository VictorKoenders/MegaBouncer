use shared::prelude::{ComponentResponse, Value};
use super::IrcChannel;
use message::Message;
use base64;

pub struct IrcServer {
    pub host: String,
    pub port: u16,
    pub nick: String,
    pub password: Option<String>,
    pub channels: Vec<IrcChannel>,
    pub buffer: String,
}

impl IrcServer {
    fn handle_message(&mut self, message: Message, response: &mut Vec<ComponentResponse>) {
        match message {
            Message::Ping(ref msg) => response.push(self.send_raw(Message::Pong(msg.clone()))),
            _ => {}
        };
        response.push(ComponentResponse::Send(::shared::Message::new_emit("irc.message", |map| {
            map.insert(String::from("host"), Value::String(self.host.clone()));
            map.insert(String::from("port"), Value::Number(self.port.into()));
            map.insert(String::from("message"), message.as_json());
        })));
    }

    pub fn handle_data(&mut self, data: String, response: &mut Vec<ComponentResponse>) {
        self.buffer += &data;
        while let Some(index) = self.buffer.bytes().position(|b| b == b'\n') {
            let line = self.buffer.drain(..index+1).collect::<String>();
            if let Some(message) = Message::from_line(&line) {
                self.handle_message(message, response);
            } else {
                println!("Could not parse line {:?}", line);
            }
        }
    }

    pub fn send_raw<T: ToString>(&self, msg: T) -> ComponentResponse {
        let str = msg.to_string() + "\r\n";
        ComponentResponse::Send(::shared::Message::new_emit("tcp.send", |map| {
            map.insert(String::from("host"), Value::String(self.host.clone()));
            map.insert(String::from("port"), Value::Number(self.port.into()));
            map.insert(String::from("data"), Value::String(base64::encode(str.to_string().as_bytes())));
        }))
    }
}