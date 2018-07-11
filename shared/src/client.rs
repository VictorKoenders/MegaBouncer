use mio::net::TcpStream;
use mio::Token;
use mio_poll_wrapper::{Handle, PollWrapper};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{Read, Write};
use {ChannelUpdate, TokenUpdate, Startup};

type StartupListener<TState> = fn(&mut Startup<TState>);
type ChannelListener<TState> = fn(&mut ChannelUpdate<TState>);
type TokenListener<TState> = fn(&mut TokenUpdate<TState>);

pub struct Client<TState: Send> {
    state: ClientState<TState>,
    stream_state: Option<StreamState>,
    write_buffer: Vec<u8>,
}

struct StreamState {
    stream: TcpStream,
    token: Token,
    is_writable: bool,
    read_buffer: Vec<u8>,
}

struct ClientState<TState: Send> {
    name: String,
    state: TState,
    listeners: HashMap<String, ChannelListener<TState>>,
    evented_listener: Option<TokenListener<TState>>,
    startup_listener: Option<StartupListener<TState>>,
}

impl<TState: Send> ClientState<TState> {
    fn json_received(&mut self, json: &Value, handle: &mut Handle) {
        let action = match json.as_object()
            .and_then(|o| o.get("action"))
            .and_then(|a| a.as_str())
        {
            Some(a) => a,
            None => {
                println!("JSON does not have an action");
                println!("{:?}", json);
                return;
            }
        };
        let mut update = ChannelUpdate {
            channel: action,
            value: json,
            state: &mut self.state,
            handle,
        };

        for listener in self.listeners.iter().filter_map(|(k, v)| {
            if ::listening_to(&[k], action) {
                Some(v)
            } else {
                None
            }
        }) {
            listener(&mut update);
        }
    }
}

impl<TState: Send + 'static> Client<TState> {
    pub fn new<T: Into<String>>(name: T, state: TState) -> Client<TState> {
        Client {
            state: ClientState {
                name: name.into(),
                state,
                listeners: HashMap::new(),
                evented_listener: None,
                startup_listener: None,
            },
            stream_state: None,
            write_buffer: Vec::with_capacity(256),
        }
    }

    pub fn register_listener<T: Into<String>>(
        &mut self,
        name: T,
        listener: ChannelListener<TState>,
    ) {
        self.state.listeners.insert(name.into(), listener);
    }

    pub fn set_token_listener(&mut self, listener: TokenListener<TState>) {
        self.state.evented_listener = Some(listener);
    }

    pub fn on_startup(&mut self, listener: StartupListener<TState>) {
        self.state.startup_listener = Some(listener);
    }

    pub fn launch(mut self) {
        loop {
            if let Err(e) = self.run() {
                println!("{}", e);
            }
            ::std::thread::sleep(::std::time::Duration::from_secs(5));
        }
    }

    fn write<T: Serialize>(&mut self, t: &T) -> Result<(), String> {
        let buff = ::serde_json::to_vec(t)
            .map_err(|e| format!("Could not convert object to json string: {:?}", e))?;
        self.write_buffer.extend_from_slice(&buff);
        self.write_buffer.extend_from_slice(b"\r\n");
        if let Some(true) = self.stream_state.as_ref().map(|s| s.is_writable) {
            self.process_write_buffer()
                .map_err(|e| format!("Could not process write buffer {:?}", e))?;
        }
        Ok(())
    }

    fn process_write_buffer(&mut self) -> Result<(), String> {
        match self.stream_state
            .as_mut()
            .unwrap()
            .stream
            .write(&self.write_buffer[..])
        {
            Ok(n) => {
                self.write_buffer.drain(..n);
                self.stream_state.as_mut().unwrap().is_writable = self.write_buffer.is_empty();
                Ok(())
            }
            Err(e) => Err(format!("Could not write to stream: {:?}", e)),
        }
    }

    fn try_process_line(&mut self, handle: &mut Handle) -> Result<(), String> {
        let stream_state = self.stream_state.as_mut().unwrap();
        while let Some(position) = stream_state.read_buffer.iter().position(|c| *c == b'\n') {
            {
                let line = ::std::str::from_utf8(&stream_state.read_buffer[..position])
                    .map_err(|e| format!("Could not read a valid utf8-string: {:?}", e))?
                    .trim();
                match ::serde_json::from_str(line) {
                    Ok(v) => self.state.json_received(&v, handle),
                    Err(e) => return Err(format!("Could not parse json: {:?}", e)),
                }
            }
            stream_state.read_buffer.drain(..position + 1);
        }
        Ok(())
    }

    fn run(&mut self) -> Result<(), String> {
        let addr = ([127u8, 0u8, 0u8, 1u8], 6142).into();
        let stream =
            TcpStream::connect(&addr).map_err(|e| format!("Could not connect to server: {:?}", e))?;

        let name = self.state.name.clone();
        self.write(&Identify(name))?;
        let keys_to_write: Vec<String> = self.state.listeners.keys().cloned().collect();
        for key in keys_to_write {
            self.write(&RegisterListener(key))?;
        }

        self.stream_state = Some(StreamState {
            stream,
            token: Token(0),
            is_writable: false,
            read_buffer: Vec::with_capacity(256),
        });

        let mut poll =
            PollWrapper::new().map_err(|e| format!("Could not create poll wrapper: {:?}", e))?;
        {
            let stream_state = self.stream_state.as_mut().unwrap();
            stream_state.token = poll.register(&stream_state.stream)
                .map_err(|e| e.to_string())?;
        }

        if let Some(startup_listener) = self.state.startup_listener {
            let mut startup = Startup {
                handle: &mut poll,
                state: &mut self.state.state,
            };
            startup_listener(&mut startup);
        }


        poll.handle(|event, handle| {
            if event.token() == self.stream_state.as_ref().unwrap().token {
                self.stream_state.as_mut().unwrap().is_writable = event.readiness().is_writable();
                if event.readiness().is_writable() && !self.write_buffer.is_empty() {
                    self.process_write_buffer()?;
                }
                if event.readiness().is_readable() {
                    loop {
                        let stream_state = self.stream_state.as_mut().unwrap();
                        let mut buffer = [0u8; 256];
                        match stream_state.stream.read(&mut buffer[..]) {
                            Ok(n) => {
                                stream_state.read_buffer.extend(&buffer[..n]);
                            }
                            Err(ref e) if e.kind() == ::std::io::ErrorKind::WouldBlock => {
                                break;
                            }
                            Err(e) => {
                                return Err(format!("Could not read from stream: {:?}", e));
                            }
                        }
                    }
                    self.try_process_line(handle)?;
                }
            } else if let Some(listener) = self.state.evented_listener {
                let mut update = TokenUpdate {
                    state: &mut self.state.state,
                    handle,
                    token: event.token(),
                    event,
                };
                listener(&mut update);
            }
            Ok(())
        })
    }
}

pub struct Identify(String);

impl Serialize for Identify {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Identify", 2)?;
        s.serialize_field("action", "node.identify")?;
        s.serialize_field("name", &self.0)?;
        s.end()
    }
}

pub struct RegisterListener(String);

impl Serialize for RegisterListener {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("RegisterListener", 2)?;
        s.serialize_field("action", "node.listener.register")?;
        s.serialize_field("channel", &self.0)?;
        s.end()
    }
}