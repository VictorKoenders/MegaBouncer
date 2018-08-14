mod server;
mod client;
mod messages;

pub use self::server::WebsocketServer;
pub use self::client::WebsocketClient;
pub use self::messages::*;

use actix_web::{ws, HttpRequest, HttpResponse, Result};
use super::ServerState;

pub fn ws_start(req: &HttpRequest<ServerState>) -> Result<HttpResponse> {
    ws::start(req, WebsocketClient::default())
}

