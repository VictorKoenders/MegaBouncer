use actix_web::fs::NamedFile;
use actix_web::{server, App, HttpRequest, Responder};
use backend::BackendRequest;
use chrono::Utc;
use state::{State, StateError};
use Result;

fn trigger_build(req: &HttpRequest) -> impl Responder {
    let project_name = req.match_info().get("project_name")?.to_string();
    let build_name = req.match_info().get("build_name")?.to_string();
    let mut error = None;
    State::get(|state| {
        if let Err(e) = state.sender.send(BackendRequest::StartBuild {
            project_name,
            build_name,
        }) {
            error = Some(e);
        }
    });
    if let Some(e) = error {
        State::modify(|state| {
            state.errors.push(StateError {
                time: Utc::now(),
                error: e.into(),
            });
        })
    }
    Some("Ok")
}
fn kill(req: &HttpRequest) -> Result<&'static str> {
    let pid: u32 = req.match_info().get("pid").unwrap_or("").parse()?;
    let mut error = None;
    State::get(|state| {
        if let Err(e) = state.sender.send(BackendRequest::KillProcess(pid)) {
            error = Some(e);
        }
    });
    if let Some(e) = error {
        State::modify(|state| {
            state.errors.push(StateError {
                time: Utc::now(),
                error: e.into(),
            });
        })
    }
    Ok("Ok")
}
fn status(_req: &HttpRequest) -> impl Responder {
    let mut result = Ok(String::new());
    State::get(|state| {
        result = ::serde_json::to_string_pretty(&state);
    });
    result
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
