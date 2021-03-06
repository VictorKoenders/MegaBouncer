extern crate mio;
extern crate mio_extras;
extern crate shared;

use mio::net::TcpListener;
use shared::mio_poll_wrapper::{Handle, PollWrapper};
use std::collections::HashMap;

fn main() {
    let mut handle = PollWrapper::new().unwrap();

    let listener = TcpListener::bind(&"0.0.0.0:8000".parse().unwrap()).unwrap();

    let process_token = handle.register(&listener).unwrap();
    let mut clients = HashMap::new();

    let result: ::std::io::Result<()> = handle.handle(|event, handle| {
        if event.token() == process_token {
            let (stream, addr) = listener.accept()?;
            println!("Accepted socket from {:?}", addr);
            let token = handle.register(&stream)?;
            clients.insert(token, stream);
        } else if let Some(client) = clients.get_mut(&event.token()) {
            println!("Received data from client {:?}", client.peer_addr());
        }
        Ok(())
    });

    if let Err(e) = result {
        println!("Could not execute: {:?}", e);
    }
}
