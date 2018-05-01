extern crate shared;
extern crate tokio;
extern crate tokio_io;
#[macro_use]
extern crate futures;
extern crate uuid;

mod client;
mod linereader;
mod server;
mod serverhandle;

use shared::serde_json;
use tokio::net::TcpListener;
use tokio::prelude::*;

pub type EmptyFuture = Box<Future<Item = (), Error = ()> + Send>;

fn main() {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let (handle, server) = server::Server::new();

    let listener = listener
        .incoming()
        .for_each(move |socket| {
            let addr = socket.peer_addr().unwrap();
            println!("accepted socket; addr={:?}", addr);
            let mut handle = handle.clone();
            let mut handle2 = handle.clone();
            let mut handle3 = handle.clone();
            let (reader, writer) = socket.split();

            let connection = linereader::LineReader::new(reader)
                .map_err(|e| {
                    println!("Could not read from client: {:?}", e);
                })
                .for_each(move |line| {
                    let result: EmptyFuture = match serde_json::from_str(&line) {
                        Ok(v) => Box::from(handle.message_received(addr, v).map_err(|e| {
                            println!("Could nto send value to central server: {:?}", e)
                        })),
                        Err(e) => {
                            println!("Could not parse JSON: '{:?}'", line);
                            println!("{:?}", e);
                            Box::from(futures::future::ok(()))
                        }
                    };
                    result
                })
                .and_then(move |_| {
                    println!("Client {:?} disconnected", addr);
                    handle2
                        .client_disconnected(addr)
                        .map_err(|e| println!("Could not send value to central server: {:?}", e))
                });

            tokio::spawn(
                handle3
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
