use super::{ActionType, Channel, Component, ComponentResponse, ComponentWrapper, Message};
use mio::{Events, Poll, PollOpt, Ready, Token};
use serde_json::{from_str, Value};
use std::collections::VecDeque;
use std::io::{Read, Write};
use mio::tcp::TcpStream;

pub struct Client {
    components: Vec<ComponentWrapper>,
    poll: Poll,
    events: Events,
    stream: Option<TcpStream>,
    message_queue: VecDeque<Message>,
    message_queue_bytes: Vec<u8>,
    incoming_buffer: String,
    writable: bool,
    running: bool,
    name: Option<String>,
}

impl Default for Client {
    fn default() -> Client {
        Client {
            components: Vec::new(),
            poll: Poll::new().unwrap(),
            events: Events::with_capacity(100),
            stream: None,
            message_queue: VecDeque::new(),
            message_queue_bytes: Vec::new(),
            incoming_buffer: String::new(),
            writable: false,
            running: true,
            name: None,
        }
    }
}

impl Client {
    pub fn register<T: 'static + Component + Default>(&mut self) {
        self.components.push(ComponentWrapper::from_component(Box::new(T::default())));
    }

    fn execute<C>(&mut self, mut callback: C)
        where C : FnMut(&mut ComponentWrapper, &Poll) -> Vec<ComponentResponse> {
        let mut components_to_add = Vec::new();
        let mut messages_to_send = VecDeque::new();
        for component in self.components.iter_mut().filter(|c| !c.removed) {
            enum ChannelDiff {
                Add(Channel),
                Remove(Channel),
            };
            let mut channel_modifications = Vec::new();
            {
                let result = callback(component, &self.poll);
                for result in result.into_iter() {
                    match result {
                        ComponentResponse::StopListeningToChannel(channel) => {
                            messages_to_send.push_back(Message::new_forget_listener(&channel));
                            channel_modifications.push(ChannelDiff::Remove(channel));
                        },
                        ComponentResponse::SpawnComponent(component) => {
                            components_to_add.push(component);
                        },
                        ComponentResponse::ListenToChannel(channel) => {
                            messages_to_send.push_back(Message::new_register_listener(&channel));
                            channel_modifications.push(ChannelDiff::Add(channel));
                        },
                        ComponentResponse::RegisterToken(token) => {
                            component.tokens.push(token);
                        },
                        ComponentResponse::RemoveToken(token) => {
                            component.tokens.retain(|t| *t != token);
                        },
                        ComponentResponse::Reply(_value) => unimplemented!(),
                        ComponentResponse::Send(message) => {
                            messages_to_send.push_back(message)
                        },
                        ComponentResponse::RemoveSelf => unimplemented!(),
                    }
                }
            }
            for channel_modification in channel_modifications.into_iter() {
                match channel_modification {
                    ChannelDiff::Add(channel) => {
                        component.channels.retain(|c| !channel.matches(c));
                        component.channels.push(channel);
                    },
                    ChannelDiff::Remove(channel) => {
                        component.channels.retain(|c| !channel.matches(c));
                    }
                }
            }
        }
        for component in components_to_add.into_iter() {
            self.components.push(ComponentWrapper::from_component(component));
        }
        let did_add_messages = messages_to_send.len() > 0;
        self.message_queue.extend(messages_to_send.into_iter());
        if did_add_messages {
            self.try_write_messages(false);
        }
    }

    fn try_write_messages(&mut self, force_writable: bool) {
        if force_writable {
            self.writable = true;
        }
        if !self.writable { return; }
        loop {
            if self.message_queue_bytes.len() > 0 {
                if let Some(ref mut stream) = self.stream {
                    match stream.write(&self.message_queue_bytes) {
                        Ok(length) => {
                            self.message_queue_bytes.drain(..length);
                            if self.message_queue_bytes.len() > 0 {
                                println!("Could not write full queue, retrying again next time");
                                self.writable = false;
                                return;
                            }
                        },
                        Err(e) => {
                            println!("Could not write to stream: {:?}", e);
                            self.writable = false;
                            return;
                        }
                    }
                }
            } else if let Some(message) = self.message_queue.pop_front() {
                self.message_queue_bytes.extend(message.to_bytes().unwrap().into_iter());
                self.message_queue_bytes.extend(b"\n");
            } else {
                break;
            }
        }
    }

    fn handle_message(&mut self, message: Message){
        match message.action {
            ActionType::Emit if message.channel.is_some() => {
                let inner_message = message.clone();
                if let Some(channel) = message.channel {
                    self.execute(|component, poll| {
                        if component.channels.iter().any(|c| c.matches(&channel)) {
                            component.component.message_received(poll, &channel, &inner_message.data)
                        } else {
                            Vec::new()
                        }
                    });
                }
            },
            ActionType::Identify => {
                match message.data.as_object().map(|o| o.get("name")) {
                    Some(Some(&Value::String(ref str))) => {
                        self.execute(|component, poll| component.component.node_connected(poll, str))
                    },
                    _ => {}
                }
            }
            _ => {
                println!("Received unknown action: {:?} ({:?} {:?})", message.action, message.channel, message.data);
            }
        }
    }

    fn try_read_message(&mut self) -> Vec<Message> {
        let mut reconnect = false;
        let mut messages = Vec::new();
        if let Some(ref mut stream) = self.stream {
            let mut buffer = [0u8;256];
            let mut has_read_data = false;
            loop {
                let length = match stream.read(&mut buffer) {
                    Err(e) => {
                        if !has_read_data {
                            println!("Could not read data: {:?}", e);
                            reconnect = true;
                            break;
                        } else {
                            return messages;
                        }
                    },
                    Ok(l) => l
                };

                if length == 0 {
                    return messages;
                }

                has_read_data = true;

                self.incoming_buffer += &String::from_utf8_lossy(&buffer[0..length]);

                while let Some(index) = self.incoming_buffer.chars().position(|c| c == '\n') {
                    let str = self.incoming_buffer.drain(0..index + 1).take(index).collect::<String>();
                    match from_str(&str).map(|json| Message::from_json(json)) {
                        Ok(Ok(message)) => {
                            messages.push(message);
                        },
                        Ok(Err(e)) => {
                            println!("Could not parse incoming JSON: {:?}", e);
                        },
                        Err(e) => {
                            println!("Could not parse incoming JSON: {:?}", e);
                        }
                    }
                }
            }
        }

        if reconnect {
            self.connect();
        }
        messages
    }

    fn connect(&mut self) {
        let addr = "127.0.0.1:12345".parse().unwrap();
        let stream = TcpStream::connect(&addr).unwrap();
        self.poll.register(&stream, Token(0), Ready::readable() | Ready::writable(), PollOpt::edge()).unwrap();
        self.stream = Some(stream);

        self.message_queue.clear();
        self.incoming_buffer.clear();
        if let Some(ref name) = self.name {
            self.message_queue.push_back(Message::new_identify(&name.clone()));
        }
        self.execute(|c, poll| c.component.init(poll));
    }

    pub fn start<T: ToString>(mut self, name: T) {
        self.name = Some(name.to_string());
        self.connect();

        while self.running {
            let count = self.poll.poll(&mut self.events, None).unwrap();
            for i in 0..count {
                let event = self.events.get(i).unwrap();
                let readiness = event.readiness();
                if event.token() == Token(0) {
                    if readiness.is_writable() {
                        self.try_write_messages(true);
                    }
                    if readiness.is_readable(){
                        let messages = self.try_read_message();
                        
                        for message in messages.into_iter(){
                            self.handle_message(message);
                        }
                    }
                } else {
                    self.execute(|component, poll| {
                        if component.tokens.iter().any(|t| *t == event.token()) {
                            component.component.token_received(poll, &event)
                        } else {
                            Vec::new()
                        }
                    });
                }
            }
        }
    }
}
