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

extern crate error_chain;
extern crate shared;
extern crate uuid;

/// holds a reference to the clients connected to the server
pub mod client;
/// Message logic
pub mod message;
/// The server logic
pub mod server;

use shared::mio::net::TcpListener;
use shared::mio_poll_wrapper::Handle;
use shared::serde_json;

fn main() {
    let addr = ([127u8, 0u8, 0u8, 1u8], 6142).into();
    let listener = TcpListener::bind(&addr).unwrap();

    let mut server = server::Server::default();

    let mut wrapper = shared::mio_poll_wrapper::PollWrapper::new().unwrap();
    let server_token = wrapper.register(&listener).unwrap();

    println!("server running on localhost:6142");
    let _: Result<(), ()> = wrapper.handle(|event, handle| {
        if event.token() == server_token {
            let (client, addr) = listener.accept().unwrap();
            let token = handle.register(&client).unwrap();
            server.add(client, addr, token);
        } else {
            server.handle(event);
        }
        Ok(())
    });
}
