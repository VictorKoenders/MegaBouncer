use actix_web::{server, App, HttpRequest, Responder};
use backend::BackendRequest;
use chrono::Utc;
use state::{State, StateError};

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
fn status(_req: &HttpRequest) -> impl Responder {
    let mut result = Ok(String::new());
    State::get(|state| {
        result = ::serde_json::to_string_pretty(&state);
    });
    result
}

pub fn run() {
    server::new(|| {
        App::new()
            .resource("/", |r| r.f(status))
            .resource("/api/build/start/{project_name}/{build_name}", |r| {
                r.f(trigger_build)
            })
    }).bind("127.0.0.1:8000")
        .expect("Can not bind to port 8000")
        .run();
}
