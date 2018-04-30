extern crate shared;
extern crate tokio;
extern crate tokio_io;
#[macro_use]
extern crate futures;

use futures::sync::mpsc::{channel, Receiver, Sender};
use shared::serde_json;
use std::net::SocketAddr;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio_io::io::WriteHalf;

fn main() {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let (handle, server) = Server::new();

    let listener = listener
        .incoming()
        .for_each(move |socket| {
            let addr = socket.peer_addr().unwrap();
            println!("accepted socket; addr={:?}", addr);
            let mut handle = handle.clone();
            let mut handle2 = handle.clone();
            let (reader, writer) = socket.split();

            let connection = LineReader::new(reader)
                .map_err(|e| {
                    println!("Could not read from client: {:?}", e);
                })
                .for_each(move |line| {
                    handle.message_received(addr.clone(), serde_json::Value::String(line))
                          .map_err(|e| {
                              println!("Could not send value to central server: {:?}", e);
                          })
                })
                .and_then(move |_| {
                    println!("Client {:?} disconnected", addr.clone());
                    Ok(())
                });

            tokio::spawn(
                handle2
                    .client_connected(addr, writer)
                    .and_then(|_| connection),
            );

            Ok(())
        })
        .map_err(|err| {
            println!("Could not listen for new clients: {:?}", err);
        });

    println!("server running on localhost:6142");
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.spawn(listener);
    runtime.spawn(server.start());
    runtime.shutdown_on_idle().wait().unwrap();
}

struct LineReader<R> {
    reader: R,
    buffer_position: usize,
    buffer: [u8; 1024],
}

impl<R> LineReader<R> {
    fn new(reader: R) -> LineReader<R> {
        LineReader {
            reader,
            buffer_position: 0,
            buffer: [0u8; 1024],
        }
    }
}

impl<R: AsyncRead> Stream for LineReader<R> {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(index) = self.buffer[..self.buffer_position]
                .iter()
                .position(|c| *c == b'\n')
            {
                let str = std::str::from_utf8(&self.buffer[..index]).unwrap().trim().to_string();
                unsafe {
                    std::ptr::copy(
                        &self.buffer[index + 1],
                        &mut self.buffer[0],
                        self.buffer_position - index - 1,
                    );
                }
                self.buffer_position -= index + 1;
                return Ok(Some(str).into());
            }
            let n = try_ready!(
                self.reader
                    .poll_read(&mut self.buffer[self.buffer_position..])
            );
            if n == 0 {
                return Ok(None.into());
            }
            self.buffer_position += n;
            if self.buffer_position == self.buffer.len() {
                println!("Buffer overflowed");
                return Ok(None.into());
            }
        }
    }
}

#[derive(Debug)]
struct Server {
    receiver: Receiver<ServerMessage>,
}
#[derive(Default, Debug)]
struct ServerState {
    clients: Vec<Client>,
}
#[derive(Clone)]
struct ServerHandle {
    sender: Sender<ServerMessage>,
}

impl ServerHandle {
    pub fn client_connected(
        &mut self,
        address: SocketAddr,
        writer: WriteHalf<TcpStream>,
    ) -> Box<Future<Item = (), Error = ()> + Send> {
        Box::new(
            self.sender
                .clone()
                .send(ServerMessage::ClientConnected(address, writer))
                .map_err(|e| {
                    println!("Could not send error to server: {:?}", e);
                })
                .map(|_| {}),
        )
    }
    pub fn message_received(
        &mut self,
        address: SocketAddr,
        value: serde_json::Value,
    ) -> Box<Future<Item = (), Error = ()> + Send> {
        Box::new(
            self.sender
                .clone()
                .send(ServerMessage::Message(address, value))
                .map_err(|e| {
                    println!("Could not send error to server: {:?}", e);
                })
                .map(|_| {}),
        )
    }
}

#[derive(Debug)]
struct Client {
    pub addres: SocketAddr,
    pub writer: WriteHalf<TcpStream>,
}

#[derive(Debug)]
pub enum ServerMessage {
    ClientConnected(SocketAddr, WriteHalf<TcpStream>),
    Message(SocketAddr, serde_json::Value),
}

impl Server {
    pub fn new() -> (ServerHandle, Server) {
        let (sender, receiver) = channel(10);

        (
            ServerHandle { sender },
            Server {
                receiver,
            }
        )
    }

    pub fn start(self) -> Box<Future<Item = (), Error = ()> + Send + 'static>{
        use std::sync::Arc;
        use std::sync::Mutex;
        let state = Arc::new(Mutex::new(ServerState {
            clients: Vec::new()
        }));
        Box::new(self.receiver.for_each(move |message| {
            let state = state.clone();
            println!("message: {:?}", message);
            println!("self: {:?}", state);
            Ok(())
        }))
    }
}

impl Future for Server {
    type Item = ();
    type Error = std::io::Error;
    fn poll(&mut self) -> Result<Async<()>, Self::Error> {
        Ok(Async::NotReady)
    }
}
