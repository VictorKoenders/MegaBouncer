extern crate actix;
extern crate actix_web;

use self::actix::{Actor, StreamHandler};
use self::actix_web::{fs::NamedFile, ws, App, HttpRequest, HttpResponse, Responder, Result};

struct WsSockets {}

struct Ws {}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
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

fn index(req: &HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body("<html><body><script src=\"launcher_ui.js\"></script></body></html>")
}

fn serve_ui_js(req: &HttpRequest) -> Result<NamedFile> {
    println!("Loading launcher_ui.js");
    NamedFile::open("../target/wasm32-unknown-unknown/release/launcher_ui.js").map_err(Into::into)
}

fn serve_ui_wasm(req: &HttpRequest) -> Result<NamedFile> {
    println!("Loading launcher_ui.wasm");
    NamedFile::open("../target/wasm32-unknown-unknown/release/launcher_ui.wasm").map_err(Into::into)
}

pub fn run() {
    let server = actix_web::server::new(|| {
        App::new()
            .resource("/", |r| r.f(index))
            .resource("/launcher_ui.js", |r| r.f(serve_ui_js))
            .resource("/launcher_ui.wasm", |r| r.f(serve_ui_wasm))
            .resource("/ws/", |r| r.f(|req| ws::start(req, Ws {})))
    }).bind("0.0.0.0:0")
        .unwrap();
    println!("Server listening on:");
    let mut first = true;
    for addr in server.addrs() {
        println!(" - {}", addr);
        if first && cfg!(debug_assertions) {
            ::std::process::Command::new("cmd")
                .arg("/C")
                .arg("start")
                .arg(format!("http://localhost:{}", addr.port()))
                .spawn()
                .unwrap();
            first = false;
        }
    }
    server.run();
}
