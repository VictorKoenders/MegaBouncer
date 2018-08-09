use actix_web::{server, App, HttpRequest, Responder};
use state::State;
use backend::BackendRequest;
use chrono::Utc;

fn trigger_build(req: &HttpRequest) -> impl Responder {
    let directory = req.match_info().get("directory")?.to_string(); 
    let name = req.match_info().get("name")?.to_string(); 
    let mut error = None;
    State::get(|state| {
        if let Err(e) = state.sender.send(BackendRequest::StartBuild { directory, name }) {
            error = Some(e);
        }
    });
    if let Some(e) = error {
        State::modify(|state| {
            state.errors.push((Utc::now(), e.into()));
        })
    }
    Some("Ok")
}
fn status(_req: &HttpRequest) -> impl Responder {
    let mut result = String::new();
    State::get(|state| {
        result = format!("{:#?}", state);
    });
    result
}

pub fn run() {
    server::new(|| {
        App::new()
            .resource("/", |r| r.f(status))
            .resource("/api/build/start/{directory}/{name}", |r| r.f(trigger_build))
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run();
}