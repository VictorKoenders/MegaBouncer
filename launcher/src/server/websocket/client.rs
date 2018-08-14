use super::{Connect, Disconnect, BroadcastStateChange};
use actix::{
    fut, Actor, ActorContext, ActorFuture, AsyncContext, ContextFutureSpawner, Handler, Running,
    StreamHandler, WrapFuture,
};
use actix_web::ws;
use server::ServerState;

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
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl Handler<BroadcastStateChange> for WebsocketClient {
    type Result = ();
    fn handle(&mut self, msg: BroadcastStateChange, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
