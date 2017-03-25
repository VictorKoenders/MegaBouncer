#![deny(warnings)]

extern crate shared;
extern crate base64;

use shared::prelude::*;

fn main() {
    let mut client = shared::Client::default();
    client.register::<IrcConnector>();
    client.start("TCP Connector");
}

#[derive(Default)]
struct IrcConnector {
    // pub servers: Vec<IrcServer>,
}

impl IrcConnector {
    fn get_join_messages(&self) -> Vec<ComponentResponse> {
        vec![
            ComponentResponse::Send(Message::new_emit("tcp.connect", |map| {
                map.insert(String::from("host"), Value::String(String::from("irc.esper.net")));
                map.insert(String::from("port"), Value::Number(6667.into()));
            })),
            ComponentResponse::Send(Message::new_emit("tcp.send", |map| {
                map.insert(String::from("host"), Value::String(String::from("irc.esper.net")));
                map.insert(String::from("port"), Value::Number(6667.into()));
                map.insert(String::from("data"), Value::String(base64::encode(b"nick TrangarRustBot\r\nuser TrangarRustBot TrangarRustBot irc.esper.net :TrangarRustBot\r\n")));
            }))
        ]
    }
    fn send_raw<T: ToString>(str: T) -> ComponentResponse {
        println!("<- {:?}", str.to_string());
        ComponentResponse::Send(Message::new_emit("tcp.send", |map| {
            map.insert(String::from("host"), Value::String(String::from("irc.esper.net")));
            map.insert(String::from("port"), Value::Number(6667.into()));
            map.insert(String::from("data"), Value::String(base64::encode(str.to_string().as_bytes())));
        }))
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
            match message.get("data").map(|v| Value::as_str(v).map(|s| base64::decode(s).map(String::from_utf8))) {
                Some(Some(Ok(Ok(line)))) => {
                    println!("-> {:?}", line);
                    if line.starts_with("PING ") {
                        let pong = format!("PONG {}", &line[5..]);
                        response.push(IrcConnector::send_raw(pong));
                    }
                },
                x => println!("Could not parse data: {:?}", x)
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


// struct IrcServer {
//     pub buffer: String,
//     pub channels: Vec<IrcChannel>,
// }

// struct IrcChannel {
//     pub name: String
// }
