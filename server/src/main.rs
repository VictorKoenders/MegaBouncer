extern crate shared;
extern crate uuid;

mod client;

use shared::mio::net::TcpListener;
use shared::mio::{Events, Poll, PollOpt, Ready, Token};

fn main() {
    let poll = Poll::new().unwrap();
    let addr = "127.0.0.1:13265".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();
    poll.register(&listener, Token(0), Ready::readable(), PollOpt::edge())
        .unwrap();
    let mut events = Events::with_capacity(100);
    let mut clients = Vec::new();
    let mut token_iterator = (1..).map(Token);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            if event.token() == Token(0) {
                let (stream, addr) = match listener.accept() {
                    Ok(s) => s,
                    Err(e) => {
                        panic!("Could not receive connections: {:?}", e);
                    }
                };
                let new_client =
                    client::Client::new(stream, addr, &poll, token_iterator.next().unwrap());
                clients.push(new_client);
            } else if let Some(index) = clients.iter().position(|c| c.is(event.token())) {
                let value = match clients[index].update(event.readiness()) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("Could not read {:?}", clients[index]);
                        println!("{:?}", e);
                        clients.remove(index);
                        continue;
                    }
                };
                println!("{:?} {:?} {:?}", clients[index], event, value);
            }
        }
    }
}
