use super::{BroadcastInitialState, BroadcastStateChange, Connect, Disconnect, WebsocketClient};
use actix::{Actor, Addr, ArbiterService, Context, Handler, Supervised};
use serde_json;
use state::{State, StateChange};
use std::collections::HashMap;

#[derive(Default)]
pub struct WebsocketServer {
    next_id: usize,
    clients: HashMap<usize, Addr<WebsocketClient>>,
}

impl Actor for WebsocketServer {
    type Context = Context<Self>;
}

impl Supervised for WebsocketServer {
    fn restarting(&mut self, _ctx: &mut Self::Context) {
        println!("WebsocketServer restarting");
    }
}

impl ArbiterService for WebsocketServer {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        println!("WebsocketServer started");
        self.next_id = 1;
    }
}

impl Handler<StateChange> for WebsocketServer {
    type Result = ();
    fn handle(&mut self, msg: StateChange, _ctx: &mut Context<Self>) -> () {
        let json = serde_json::to_string(&msg).expect("Could not serialize StateChange");
        for client in self.clients.values() {
            client.do_send(BroadcastStateChange(json.clone()));
        }
    }
}

impl Handler<Connect> for WebsocketServer {
    type Result = usize;
    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> usize {
        let id = self.next_id;
        let _ = State::get(|state| {
            let json = serde_json::to_string(&state)?;
            msg.client_addr.do_send(BroadcastInitialState(json));
            Ok(())
        });
        self.clients.insert(id, msg.client_addr);
        println!("Accepting incoming client {}", id);
        self.next_id += 1;
        id
    }
}

impl Handler<Disconnect> for WebsocketServer {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _ctx: &mut Context<Self>) -> () {
        println!("Dropping client {}", msg.id);
        self.clients.remove(&msg.id);
    }
}
