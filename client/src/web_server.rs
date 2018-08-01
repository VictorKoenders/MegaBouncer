use actix::*;
use actix_web::fs::NamedFile;
use actix_web::http::Method;
use actix_web::{ws, App, HttpRequest, Result};
use futures::future::Future;
use std::thread::{sleep_ms, spawn};

const INDEX_FILE: &str = "ui/index.html";
const DIST_FILE: &str = "ui/dist/bundle.js";
const DIST_MAP_FILE: &str = "ui/dist/bundle.js.map";

struct Ws;

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("{:?}", msg);
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            _ => (),
        }
    }
}

fn index(_req: &HttpRequest) -> Result<NamedFile> {
    Ok(NamedFile::open(INDEX_FILE)?)
}

fn bundle(_req: &HttpRequest) -> Result<NamedFile> {
    Ok(NamedFile::open(DIST_FILE)?)
}

fn bundle_map(_req: &HttpRequest) -> Result<NamedFile> {
    Ok(NamedFile::open(DIST_MAP_FILE)?)
}

pub fn serve() -> String {
    let mut system = ::actix::System::new("MegaBouncer client");

    let addr: ::std::net::SocketAddr = ([127, 0, 0, 1], 0).into();
    let server = ::actix_web::server::new(|| {
        App::new()
            .resource("/", |r| r.method(Method::GET).f(index))
            .resource("/bundle.js", |r| r.method(Method::GET).f(bundle))
            .resource("/bundle.js.map", |r| r.method(Method::GET).f(bundle_map))
            .resource("/ws/", |r| r.f(|req| ws::start(req, Ws)))
    }).bind(addr)
        .unwrap();
    let addr = server.addrs()[0];
    let url = format!("http://{}", addr);

    server.run();
    println!("Server listening on {}", addr);
    while !server_is_up(&mut system, url.clone()) {
        sleep_ms(1000);
    }
    url
}

fn server_is_up(system: &mut SystemRunner, url: String) -> bool {
    system
        .block_on(
            ::actix_web::client::get(url)
            .finish()
            .unwrap()
            .send()                               // <- Send http request
            .and_then(|response| {                // <- server http response
                ::futures::future::ok(response.status().is_success())
            }),
        )
        .unwrap()
}
