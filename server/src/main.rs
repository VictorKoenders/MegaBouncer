#![deny(missing_docs)]

//! Server module for the MegaBouncer Project
//! This application starts up a server on port 6142 and listens to incoming connections
//! Incoming connections can communicate with the server through a newline JSON protocol
//!
//! ## Messages
//! Every line should be a valid JSON object with an "action" field
//! This field can be one of the following values:
//! ### node.identify
//! This must be called as the first action, and requires a field "name"
//! The node will be named this way
//! ### node.listener.register
//! Registers a listener for the current node. This means that any message that is broadcasted, and matches this pattern, the node will be notified.
//!
//! Listeners can use subgroups and wildcards, e.g.:
//! - "test" matches "test", "test.test" and "test.asd"
//! - "test.*" matches "test.test" and "test.asd", but not "test"
//! - "test.asd" only matches "test.asd", not "test" and "test.test"
//!
//! ### node.list
//! Returns a list of nodes. Each node has an id, a name, and a list of channels that it is subscribed to
//!
//! ### other
//! Any other value will be broadcasted to all nodes, if they are listening to it.
//!
//! ## Additional broadcasts
//! The server will create additional broadcasts based on the connectivity of the nodes
//!
//! ### node.identified
//! Is generated when a node succesfully identifies
//!
//! ### node.disconnected
//! Is generated when a TCP connection of a node (with a name) is dropped
//!
//! ### node.listener.registered
//! Is generated when a node starts listening to a channel

extern crate shared;
extern crate uuid;

/// holds a reference to the clients connected to the server
pub mod client;
/// The server logic
pub mod server;
/// A handle that can be passed around between futures and that will send values to the central server
pub mod serverhandle;

use shared::futures;
use shared::futures::{Future, Stream};
use shared::linereader;
pub use shared::serde_json;
use shared::tokio;
use shared::tokio::net::{TcpListener, TcpStream};
use shared::tokio_io::AsyncRead;

fn main() {
    let addr = ([127u8, 0u8, 0u8, 1u8], 6142).into();
    let listener = TcpListener::bind(&addr).unwrap();

    let (handle, server) = server::Server::new();

    let listener = listener
        .incoming()
        .map_err(|err| {
            println!("Could not listen for new clients: {:?}", err);
        })
        .for_each(move |socket| {
            spawn_client(socket, &handle.clone());
            Ok(())
        });

    println!("server running on localhost:6142");
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.spawn(listener);
    runtime.spawn(server.start());
    runtime.shutdown_on_idle().wait().unwrap();
}

/// Spawn a client with a given TcpStream and handle to the server
fn spawn_client(socket: TcpStream, handle: &serverhandle::ServerHandle) {
    let addr = socket.peer_addr().unwrap();
    println!("accepted socket; addr={:?}", addr);
    let mut handle = handle.clone();
    let mut handle2 = handle.clone();
    let mut handle3 = handle.clone();
    let mut handle4 = handle.clone();
    let (reader, writer) = socket.split();

    let connection = linereader::LineReader::new(reader)
        .map_err(move |e| {
            println!("Could not read from client: {:?}", e);
            tokio::spawn(
                handle.client_disconnected(addr)
                .map_err(|e| println!("Could not send value to central server"))
            );
        })
        .for_each(move |line| {
            let result: shared::EmptyFuture = match serde_json::from_str(&line) {
                Ok(v) => Box::from(
                    handle2
                        .message_received(addr, v)
                        .map_err(|e| println!("Could nto send value to central server: {:?}", e)),
                ),
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
            handle3
                .client_disconnected(addr)
                .map_err(|e| println!("Could not send value to central server: {:?}", e))
        });

    tokio::spawn(
        handle4
            .client_connected(addr, writer)
            .and_then(|_| connection),
    );
}
