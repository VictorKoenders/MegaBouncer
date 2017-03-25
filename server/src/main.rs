#![deny(warnings)]

extern crate serde_json;
extern crate rangetree;
extern crate shared;
extern crate time;
extern crate mio;

mod client;
mod server;

fn main() {
    let server = server::Server::new("127.0.0.1", 12345).unwrap();
    server.run().unwrap();
}
