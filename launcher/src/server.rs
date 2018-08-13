use actix_web::fs::NamedFile;
use actix_web::{server, App, HttpRequest, Responder};
use backend::BackendRequest;
use state::State;
use std::sync::mpsc::channel;
use std::thread::spawn;
use Result;

fn trigger_build(req: &HttpRequest) -> impl Responder {
    let project_name = req
        .match_info()
        .get("project_name")
        .unwrap_or("")
        .to_string();
    let build_name = req.match_info().get("build_name").unwrap_or("").to_string();
    State::get(|state| {
        state
            .backend_sender
            .send(BackendRequest::StartBuild {
                project_name,
                build_name,
            }).map_err(Into::into)
    }).map(|_| "Ok")
}
fn kill(req: &HttpRequest) -> Result<&'static str> {
    let pid: u32 = req.match_info().get("pid").unwrap_or("").parse()?;
    State::get(|state| {
        state
            .backend_sender
            .send(BackendRequest::KillProcess(pid))
            .map_err(Into::into)
    }).map(|_| "Ok")
}
fn status(_req: &HttpRequest) -> impl Responder {
    State::get(|state| ::serde_json::to_string_pretty(&state).map_err(Into::into))
}

fn index(_req: &HttpRequest) -> impl Responder {
    NamedFile::open("ui/index.html")
}

fn bundle(_req: &HttpRequest) -> impl Responder {
    NamedFile::open("ui/dist/bundle.js")
}

fn bundle_map(_req: &HttpRequest) -> impl Responder {
    NamedFile::open("ui/dist/bundle.js.map")
}

pub fn run() {
    let (sender, receiver) = channel();
    State::set_change_sender(sender);
    spawn(move || {
        while let Ok(item) = receiver.recv() {
            println!("Change: {:?}", item);
        }
    });
    server::new(|| {
        App::new()
            .resource("/", |r| r.f(index))
            .resource("/bundle.js", |r| r.f(bundle))
            .resource("/bundle.js.map", |r| r.f(bundle_map))
            .resource("/api/state", |r| r.f(status))
            .resource("/api/kill/{pid}", |r| r.f(kill))
            .resource("/api/build/start/{project_name}/{build_name}", |r| {
                r.f(trigger_build)
            })
    }).bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run();
}
