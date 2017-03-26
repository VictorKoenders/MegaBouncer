use shared::prelude::*;
use super::{IrcChannel, IrcServer};
use base64;

pub struct IrcConnector {
    pub servers: Vec<IrcServer>,
}

impl Default for IrcConnector {
    fn default() -> IrcConnector {
        IrcConnector {
            servers: vec![
                IrcServer {
                    host: String::from("irc.esper.net"),
                    port: 6667,
                    nick: String::from("TrangarRustBot"),
                    password: None,
                    buffer: String::new(),
                    channels: vec![
                        IrcChannel {
                            name: String::from("#trangarbot")
                        }
                    ]
                }
            ]
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
        ];
        vec.append(&mut self.get_join_messages());
        vec
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
        } else {
            println!("Received {:?}: {:?}", channel, message);
        }
        response
    }
    fn node_connected(&mut self, _poll: &Poll, name: &String) -> Vec<ComponentResponse>{
        if name == "TCP Connector" {
            self.get_join_messages()
        } else {
            Vec::new()
        }
    }
}
