use super::{BroadcastInitialState, BroadcastStateChange, Connect, Disconnect};
use actix::{
    fut, Actor, ActorContext, ActorFuture, AsyncContext, ContextFutureSpawner, Handler, Running,
    StreamHandler, WrapFuture,
};
use actix_web::ws;
use backend::BackendRequest;
use serde_json::{self, Value};
use server::ServerState;
use state::State;

#[derive(Default)]
pub struct WebsocketClient {
    id: usize,
}

impl Actor for WebsocketClient {
    type Context = ws::WebsocketContext<Self, ServerState>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let client_addr = ctx.address();
        ctx.state()
            .websocket_server
            .send(Connect { client_addr })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(id) => act.id = id,
                    _ => ctx.stop(),
                }
                fut::ok(())
            }).wait(ctx)
    }
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        ctx.state()
            .websocket_server
            .do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WebsocketClient {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                let data: Value = match serde_json::from_str(&text) {
                    Ok(d) => d,
                    Err(e) => {
                        State::report_error(&format_err!(
                            "Could not parse json from client. {:?}",
                            e
                        ));
                        return;
                    }
                };
                if let Value::Number(n) = &data["kill"] {
                    if let Some(pid) = n.as_u64() {
                        let _ = State::get(|state| {
                            state
                                .backend_sender
                                .send(BackendRequest::KillProcess(pid as u32))
                                .map_err(Into::into)
                        });
                    }
                } else if let Value::Array(arr) = &data["start"] {
                    if let (Some(project_name), Some(build_name)) = (
                        arr.get(0).and_then(Value::as_str),
                        arr.get(1).and_then(Value::as_str),
                    ) {
                        let _ = State::get(|state| {
                            state
                                .backend_sender
                                .send(BackendRequest::StartBuild {
                                    project_name: project_name.to_string(),
                                    build_name: build_name.to_string(),
                                }).map_err(Into::into)
                        });
                    }
                } else {
                    State::report_error(&format_err!("Unknown json from websocket client"));
                }
            }
            ws::Message::Binary(_bin) => {
                println!("Unknown binary blob from client, ignoring");
            }
            ws::Message::Pong(_msg) => {}
            ws::Message::Close(msg) => {
                println!("Client closed: {:?}", msg);
                ctx.close(None);
            }
        }
    }
}

impl Handler<BroadcastStateChange> for WebsocketClient {
    type Result = ();
    fn handle(&mut self, msg: BroadcastStateChange, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl Handler<BroadcastInitialState> for WebsocketClient {
    type Result = ();
    fn handle(&mut self, msg: BroadcastInitialState, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
