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
            let (reader, writer) = socket.split();

            let connection = linereader::LineReader::new(reader)
                .map_err(|e| {
                    println!("Could not read from client: {:?}", e);
                })
                .for_each(move |line| {
                    handle
                        .message_received(addr, serde_json::Value::String(line))
                        .map_err(|e| {
                            println!("Could not send value to central server: {:?}", e);
                        })
                })
                .and_then(move |_| {
                    println!("Client {:?} disconnected", addr);
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
