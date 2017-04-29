use shared::prelude::*;
use super::{/*IrcChannel,*/ IrcServer};
use base64;

pub struct IrcConnector {
    pub servers: Vec<IrcServer>,
}

impl Default for IrcConnector {
    fn default() -> IrcConnector {
        IrcConnector {
            servers: Vec::new()
        }
    }
}

impl IrcConnector {
    fn get_join_messages(&self) -> Vec<ComponentResponse> {
        let mut response = Vec::new();
        for server in &self.servers {
            response.push(ComponentResponse::Send(Message::new_emit("tcp.connect", |map| {
                map.insert(String::from("host"), Value::String(server.host.clone()));
                map.insert(String::from("port"), Value::Number(server.port.into()));
            })));
            let auth_string = format!("nick {0}\r\nuser {0} {0} {1} :{0}\r\n", server.nick, server.host);
            response.push(ComponentResponse::Send(Message::new_emit("tcp.send", |map| {
                map.insert(String::from("host"), Value::String(server.host.clone()));
                map.insert(String::from("port"), Value::Number(server.port.into()));
                map.insert(String::from("data"), Value::String(base64::encode(auth_string.as_bytes())));
            })));
        }
        response
    }
    // fn send_raw<T: ToString>(server: &IrcServer, str: T) -> ComponentResponse {
    //     println!("<- {:?}", str.to_string());
    //     ComponentResponse::Send(Message::new_emit("tcp.send", |map| {
    //         map.insert(String::from("host"), Value::String(server.host.clone()));
    //         map.insert(String::from("port"), Value::Number(server.port.into()));
    //         map.insert(String::from("data"), Value::String(base64::encode(str.to_string().as_bytes())));
    //     }))
    // }

    fn get_message_data(value: &Value) -> Option<String> {
        match value.get("data") {
            Some(&Value::String(ref data)) => match base64::decode(data) {
                Ok(data) => match String::from_utf8(data) {
                    Ok(line) => Some(line),
                    _ => None
                },
                _ => None
            },
            _ => None
        }
    }

    fn get_server(&mut self, value: &Value) -> Option<&mut IrcServer> {
        match value.get("host"){
            Some(&Value::String(ref host)) => self.servers.iter_mut().find(|s| s.host == *host),
            _ => None
        }
    }
    
    fn send_message(&mut self, message: &Value, response: &mut Vec<ComponentResponse>) {
        let server: &mut IrcServer = match self.get_server(message) {
            Some(server) => server,
            None => {
                println!("Could not find message server: {:?}", message);
                return;
            }
        };
        
        let message_type: &String = match message.as_object().map(|o| o.get("type")) {
            Some(Some(&Value::String(ref t))) => t,
            _ => {
                println!("Could not find message type: {:?}", message);
                return;
            }
        };
        match message_type.as_str() {
            "privmsg" => {
                let target: &String = match message.as_object().map(|o| o.get("target")) {
                    Some(Some(&Value::String(ref t))) => t,
                    _ => {
                        println!("Could not find privmsg target: {:?}", message);
                        return;
                    }
                };
                let message: &String = match message.as_object().map(|o| o.get("message")) {
                    Some(Some(&Value::String(ref t))) => t,
                    _ => {
                        println!("Could not find privmsg message: {:?}", message);
                        return;
                    }
                };
                response.push(server.send_raw(format!("PRIVMSG {} :{}", target, message)));
            },
            _ => println!("Unknown type {:?}: {:?}", message_type, message)
        };
    }
}

impl Component for IrcConnector {
    fn init(&self, _poll: &Poll) -> Vec<ComponentResponse> {
        let mut vec = vec![
            ComponentResponse::ListenToChannel(Channel::from_string("irc.send")),
            ComponentResponse::ListenToChannel(Channel::from_string("irc.server.connect")),
            ComponentResponse::ListenToChannel(Channel::from_string("irc.server.disconnect")),
            ComponentResponse::ListenToChannel(Channel::from_string("irc.channel.join")),
            ComponentResponse::ListenToChannel(Channel::from_string("irc.channel.leave")),
            ComponentResponse::ListenToChannel(Channel::from_string("tcp.connected")),
            ComponentResponse::ListenToChannel(Channel::from_string("tcp.data")),
            ComponentResponse::ListenToChannel(Channel::from_string("tcp.disconnected")),
            ComponentResponse::Send(Message::new_emit_with_id("data.get_by_key", |mut map| {
                map.insert(String::from("key"), Value::String(String::from("irc.config")));
            })),
        ];
        vec.append(&mut self.get_join_messages());
        vec
    }

    fn reply_received(&mut self, _poll: &Poll, _uuid: Uuid, channel: &Option<Channel>, message: &Value) -> Vec<ComponentResponse> {
        let key = match message.as_object().and_then(|o| o.get("key")) {
            Some(&Value::String(ref key)) if key == "irc.config" => { println!("IRC Config!"); key },
            Some(&Value::String(ref key)) => { println!("Unknown key: {:?}", key); key },
            Some(x) => { println!("Unknown reply: {:?}", x); return Vec::new(); },
            None => { println!("Reply has no key: {:?}", message); return Vec::new(); }
        };
        println!("Got reply: {:?} {:?} {:?}", channel, key, message);
        Vec::new()
    }

    fn message_received(&mut self, _poll: &Poll, channel: &Channel, message: &Value) -> Vec<ComponentResponse>{
        let mut response = Vec::new();
        if channel == "tcp.data" {
            match IrcConnector::get_message_data(message) {
                Some(line) => {
                    match self.get_server(message) { 
                        Some(ref mut server) => {
                            server.handle_data(line, &mut response);
                        },
                        None => {
                            println!("Could not find server: {:?}", message);
                        }
                    }
                },
                None => {
                    println!("Could not find message data: {:?}", message);
                }
            };
        } else if channel == "irc.send" {
            self.send_message(message, &mut response);
        } else {
            println!("Received {:?}: {:?}", channel, message);
        }
        response
    }
    fn node_connected(&mut self, _poll: &Poll, name: &str) -> Vec<ComponentResponse>{
        if name == "TCP Connector" {
            self.get_join_messages()
        } else {
            Vec::new()
        }
    }
}
