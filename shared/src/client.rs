use mio::net::TcpStream;
use mio::{Events, Poll, PollOpt, Ready, Token};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{Read, Write};

type Listener<TState> = fn(&mut TState, &str, &Value);

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
    listeners: HashMap<String, Listener<TState>>,
}

impl<TState: Send> ClientState<TState> {
    fn json_received(&mut self, json: Value) {
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

        for listener in self.listeners.iter().filter_map(|(k, v)| {
            if ::listening_to(&[k], action) {
                Some(v)
            } else {
                None
            }
        }) {
            listener(&mut self.state, action, &json);
        }
    }
}

impl<TState: Send + 'static> Client<TState> {
    pub fn new<T: Into<String>>(name: T, state: TState) -> Client<TState> {
        Client {
            state: ClientState {
                name: name.into(),
                state: state,
                listeners: HashMap::new(),
            },
            stream_state: None,
            write_buffer: Vec::with_capacity(256),
        }
    }

    pub fn register_listener<T: Into<String>>(&mut self, name: T, listener: Listener<TState>) {
        self.state.listeners.insert(name.into(), listener);
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
        let mut buff = ::serde_json::to_vec(t)
            .map_err(|e| format!("Could not convert object to json string: {:?}", e))?;
        self.write_buffer.extend_from_slice(&mut buff);
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
                self.stream_state.as_mut().unwrap().is_writable = self.write_buffer.len() == 0;
                Ok(())
            }
            Err(e) => Err(format!("Could not write to stream: {:?}", e)),
        }
    }

    fn try_process_line(&mut self) -> Result<(), String> {
        let stream_state = self.stream_state.as_mut().unwrap();
        while let Some(position) = stream_state.read_buffer.iter().position(|c| *c == b'\n') {
            {
                let line = ::std::str::from_utf8(&stream_state.read_buffer[..position])
                    .map_err(|e| format!("Could not read a valid utf8-string: {:?}", e))?
                    .trim();
                match ::serde_json::from_str(line) {
                    Ok(v) => self.state.json_received(v),
                    Err(e) => return Err(format!("Could not parse json: {:?}", e))
                }
            }
            stream_state.read_buffer.drain(..position + 1);
        }
        Ok(())
    }

    fn run(&mut self) -> Result<(), String> {
        let addr = ([127u8, 0u8, 0u8, 1u8], 6142).into();
        let token = Token(1);
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
            token,
            is_writable: false,
            read_buffer: Vec::with_capacity(256),
        });
        let mut events = Events::with_capacity(10);
        let poll = Poll::new().map_err(|e| format!("Could not create poll: {:?}", e))?;
        poll.register(
            &self.stream_state.as_ref().unwrap().stream,
            self.stream_state.as_ref().unwrap().token,
            Ready::all(),
            PollOpt::edge(),
        ).map_err(|e| format!("Could not register stream: {:?}", e))?;
        while poll.poll(&mut events, None)
            .map_err(|e| format!("Could not poll: {:?}", e))? > 0
        {
            for event in &events {
                println!("{:?}", event);
                if event.token() == self.stream_state.as_ref().unwrap().token {
                    self.stream_state.as_mut().unwrap().is_writable =
                        event.readiness().is_writable();
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
                                Err(ref e) if e.kind() == ::std::io::ErrorKind::WouldBlock => break,
                                Err(e) => {
                                    return Err(format!("Could not read from stream: {:?}", e));
                                }
                            }
                        }
                        self.try_process_line()?;
                    }
                }
            }
        }
        Ok(())
    }

    /*
    fn run(&mut self) -> ::EmptyFuture {
        let addr = ([127u8, 0u8, 0u8, 1u8], 6142).into();
        let state = self.state.clone();
        Box::from(
            TcpStream::connect(&addr)
                .map_err(|e| {
                    println!("Could not connected to server");
                    println!("{:?}", e);
                })
                .and_then(move |mut stream| {
                    let state = state.clone();
                    println!("Connected to server");
                    stream.write_async(&Identify(state.lock().unwrap().name.clone()));
                    for listener in state.lock().unwrap().listeners.keys() {
                        stream.write_async(&RegisterListener(listener.clone()));
                    }
                    let reader = ::linereader::LineReader::new(stream);
                    reader
                        .for_each(move |line| {
                            let state = state.clone();
                            match ::serde_json::from_str(&line) {
                                Ok(v) => {
                                    println!("{:?}", v);
                                    let actions = state.lock().unwrap().json_received(v);
                                }
                                Err(e) => {
                                    println!("Could not parse JSON: {:?}", e);
                                    println!("{:?}", line);
                                }
                            }
                            Ok(())
                        })
                        .map_err(|e| {
                            println!("Could not read from server");
                            println!("{:?}", e);
                        })
                }),
        )
    }
    */
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
