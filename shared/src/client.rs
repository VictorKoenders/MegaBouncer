use std::path::Path;
use mio::net::TcpStream;
use mio::Token;
use mio_poll_wrapper::{Handle, PollWrapper};
use serde::Serialize;
use client_state::ClientState;
use std::io::{Read, Write};
use {ChannelUpdate, TokenUpdate, Startup};
use messages::{Identify, RegisterListener};

pub type StartupListener<TState> = fn(&mut Startup<TState>);
pub type ChannelListener<TState> = fn(&mut ChannelUpdate<TState>);
pub type TokenListener<TState> = fn(&mut TokenUpdate<TState>);

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

impl<TState: Send + 'static> Client<TState> {
    pub fn new<T: Into<String>>(name: T, state: TState) -> Client<TState> {
        Client {
            state: ClientState::new(name.into(), state),
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

    pub fn register_user_interface(&mut self, javascript: impl AsRef<Path>) {
        let mut str = String::new();
        ::std::fs::File::open(javascript.as_ref()).unwrap().read_to_string(&mut str).unwrap();
        self.state.user_interface = Some(str);
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
        self.poll_handle(poll)
    }

    fn poll_handle(&mut self, poll: PollWrapper)  -> Result<(), String> {
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
