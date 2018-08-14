mod routes;
mod websocket;

use self::routes::{bundle, bundle_map, index, kill, status, trigger_build};
use self::websocket::{ws_start, WebsocketServer};
use actix::{Addr, ArbiterService, System};
use actix_web::{server, App};
use state::State;

pub struct ServerState {
    pub websocket_server: Addr<WebsocketServer>,
}

pub fn run() {
    let _runner = System::new("Launcher");
    let server = WebsocketServer::start_service();
    State::set_change_sender(server.clone().recipient());

    server::new(move || {
        App::with_state(ServerState {
            websocket_server: server.clone(),
        }).resource("/", |r| r.f(index))
        .resource("/bundle.js", |r| r.f(bundle))
        .resource("/bundle.js.map", |r| r.f(bundle_map))
        .resource("/api/state", |r| r.f(status))
        .resource("/api/kill/{pid}", |r| r.f(kill))
        .resource("/api/build/start/{project_name}/{build_name}", |r| {
            r.f(trigger_build)
        }).resource("/ws", |r| r.f(ws_start))
    }).bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run();
}
