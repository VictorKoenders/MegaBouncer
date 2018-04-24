extern crate shared;

use shared::mio::{Poll, Token, Ready, PollOpt, Events};
use shared::mio::net::TcpListener;

fn main() {
    let poll = Poll::new().unwrap();
    let addr = "127.0.0.1:13265".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();
    poll.register(&listener, Token(0), Ready::readable(), PollOpt::edge()).unwrap();
    let mut events = Events::with_capacity(100);
    let mut clients = Vec::new();
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            println!("{:?}", event.token());
            if event.token() == Token(0) {
                let (stream, addr) = match listener.accept() {
                    Ok(s) => s,
                    Err(e) => {
                        panic!("Could not receive connections: {:?}", e);
                    }
                };
                println!("Received connection from {:?}", addr);
                let new_client = shared::Reader::new(stream, &poll, Token(clients.len() + 1));
                clients.push(new_client);
            } else {
                if let Some(index) = clients.iter().position(|c| c.is(event.token())) {
                    println!("Received data for client {:?}", clients[index]);
                    match clients[index].read() {
                        Ok(v) => println!("{:?}", v),
                        Err(e) => {
                            println!("{:?}", e);
                            println!("Disconnecting");
                            clients.remove(index);
                        }
                    }
                }
            }
        }
    }
}
