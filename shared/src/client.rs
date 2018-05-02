use futures::{Future, Stream};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpStream;
use EmptyFuture;

type Listener<TState> = fn(&mut TState, &str, &Value);

pub struct Client<TState: Send> {
    state: Arc<Mutex<ClientState<TState>>>,
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
            state: Arc::new(Mutex::new(ClientState {
                name: name.into(),
                state: state,
                listeners: HashMap::new(),
            })),
        }
    }

    pub fn register_listener<T: Into<String>>(&mut self, name: T, listener: Listener<TState>) {
        self.state
            .lock()
            .unwrap()
            .listeners
            .insert(name.into(), listener);
    }

    pub fn launch(mut self) {
        loop {
            tokio::run(self.run());
            ::std::thread::sleep(::std::time::Duration::from_secs(5));
        }
    }

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
}

pub trait Writer {
    fn write_async<T: Serialize>(&mut self, value: &T) -> EmptyFuture;
}

impl Writer for TcpStream {
    fn write_async<T: Serialize>(&mut self, value: &T) -> EmptyFuture {
        let val = ::serde_json::to_vec(value.into()).unwrap();
        self.write_all(&val).unwrap();
        self.write_all(b"\r\n").unwrap();
        self.flush().unwrap();
        Box::new(::futures::future::ok(()))
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
