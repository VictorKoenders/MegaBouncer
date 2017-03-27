use mio::{Events, Token, Poll, PollOpt, Ready};
use shared::{ActionType, Error, Result};
use client::{Client, ClientEvent};
use std::collections::HashMap;
use mio::tcp::TcpListener;
use rangetree::RangeTree;
//use mio::timer::Timer;

const SERVER_TOKEN: Token = Token(0);

pub struct Server {
    poll: Poll,
    events: Events,
    server: TcpListener,
    clients: HashMap<Token, Client>,
    rangetree: RangeTree<usize>,
    //timer: Timer<()>,
}

pub fn get_time() -> String {
    let time = ::time::now();
    format!("[{:02}:{:02}:{:02}]",
            time.tm_hour,
            time.tm_min,
            time.tm_sec)
}

impl Server {
    pub fn new(host: &str, port: u16) -> Result<Server> {
        let address = &format!("{}:{}", host, port).parse()?;
        let listener = TcpListener::bind(address)?;
        let poll = Poll::new()?;
        let events = Events::with_capacity(1024);
        //let timer = Timer::default();
        let server = Server {
            poll: poll,
            events: events,
            server: listener,
            clients: HashMap::new(),
            rangetree: RangeTree::new([1, usize::max_value()], false),
            //timer: timer,
        };
        Ok(server)
    }

    fn log_error<E: ::std::error::Error>(&mut self, error: E) {
        println!("{} Error: {:?}", get_time(), error);
    }

    fn log_client_error<E: ::std::error::Error>(&mut self, client: Client, error: E) {
        println!("{} Client {:?} error: {:?}", get_time(), client.address, error);
    }

    pub fn run(mut self) -> Result<()> {
        self.poll.register(&self.server, SERVER_TOKEN, Ready::readable() | Ready::writable(), PollOpt::edge())?;

        loop {
            let mut errors = Vec::new();
            let size = self.poll.poll(&mut self.events, None)?;
            for i in 0..size {
                let event = self.events
                    .get(i)
                    .ok_or_else(|| Error::new_could_not_get_index(i, "Event list"))?;
                //println!("{} {} {:?}", get_time(), i, event);

                let mut events = None;
                if event.token() == SERVER_TOKEN {
                    let (sock, addr) = self.server.accept()?;
                    println!("{} Received client {:?} {:?}", get_time(), sock, addr);
                    let token =
                        Token(self.rangetree
                                  .take_any()
                                  .ok_or_else(|| {
                                                  Error::new_could_not_get_index(0, "Range tree")
                                              })?);
                    self.poll.register(&sock, token, Ready::readable() | Ready::writable(), PollOpt::edge())?;

                    let client = Client::new(sock, addr);
                    self.clients.insert(token, client);
                } else if let Some(ref mut client) = self.clients.get_mut(&event.token()) {
                    let readiness = event.readiness();
                    client.set_writable(readiness.is_writable());
                    client.set_readable(readiness.is_readable());
                    if readiness.is_readable() {
                        events = Some(match client.read_data() {
                            Ok(ev) => ev,
                            Err(e) => {
                                errors.push((e, event.token()));
                                continue;
                            }
                        });
                    }
                } else {
                    println!("{} Client with token {:?} not found",
                             get_time(),
                             event.token());
                }

                if let Some(events) = events {
                    self.handle_client_events(&event.token(), events);
                }
            }

            for error in errors {
                if let Some(client) = self.clients.remove(&error.1) {
                    self.log_client_error(client, error.0);
                } else {
                    self.log_error(error.0);
                }
            }
        }
    }

    // fn emit(&mut self, message: Message) {
    //     for ref mut client in self.clients.values_mut() {
    //         client.try_send(message.clone());
    //     }
    // }

    fn handle_client_events(&mut self, token: &Token, events: Vec<ClientEvent>) {
        for event in events {
            match event {
                ClientEvent::Disconnect => {
                    if let Some(client) = self.clients.remove(token) {
                        println!("{} Client {:?} disconnected", get_time(), client.address);
                    } else {
                        println!("{} Client {:?} disconnected", get_time(), token);
                    }
                    self.rangetree.release(token.0);
                }
                ClientEvent::Broadcast(message) => {
                    if let Some(channel) = message.channel.clone() {
                        for client in self.clients.iter_mut().map(|c| c.1) {
                            if message.action != ActionType::Emit || client.is_listening_to(&channel) {
                                client.write(message.clone());
                            }
                        }
                    } else {
                        println!("Emitting message without a channel: {:?}", message);
                    }
                }
            }
        }
    }
}
