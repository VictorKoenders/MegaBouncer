mod client;
mod messages;
mod server;

pub use self::client::WebsocketClient;
pub use self::server::WebsocketServer;
// TODO: Replace this wildcard by the actual messages we use
pub use self::messages::*;

use super::ServerState;
use actix_web::{ws, HttpRequest, HttpResponse, Result};

pub fn ws_start(req: &HttpRequest<ServerState>) -> Result<HttpResponse> {
    ws::start(req, WebsocketClient::default())
}
