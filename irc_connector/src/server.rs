use message::Message;
use shared::prelude::{ComponentResponse, Value};
use super::IrcChannel;
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
    pub fn from_json_array(value: &Value) -> Vec<IrcServer> {
        let arr = match *value {
            Value::Array(ref arr) => arr,
            _ => return Vec::new()
        };

        let mut result = Vec::new();

        for item in arr {
            if let Some(server) = IrcServer::from_json(item) {
                result.push(server);
            }
        }

        result
    }

    fn from_json(value: &Value) -> Option<IrcServer> {
        let host: String = match value.get("host").and_then(Value::as_str) { Some(s) => s.to_string(), None => return None };
        let port: u16 = value.get("port").and_then(Value::as_u64).unwrap_or_else(||6667) as u16;
        let nick: String = match value.get("nick").and_then(Value::as_str) { Some(s) => s.to_string(), None => return None };
        let password: Option<String> = value.get("password").and_then(Value::as_str).map(String::from);

        let mut channels = Vec::new();

        if let Some(&Value::Array(ref arr)) = value.get("channels") {
            for channel in arr {
                if let Some(channel) = IrcChannel::from_json(channel) {
                    channels.push(channel);
                }
            }
        }
        
        Some(IrcServer {
            host: host,
            port: port,
            nick: nick,
            password: password,
            channels: channels,
            buffer: String::new(),
        })
    }

    fn handle_message(&mut self, message: Message, response: &mut Vec<ComponentResponse>) {
        if let Message::Ping(ref msg) = message {
            response.push(self.send_raw(Message::Pong(msg.clone())));
        }
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