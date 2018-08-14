use super::ServerState;
use actix_web::{fs::NamedFile, HttpRequest, Responder};
use state::State;
use serde_json;

/*
#[deprecated(note = "Use websockets instead")]
pub fn trigger_build(req: &HttpRequest<ServerState>) -> impl Responder {
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

#[deprecated(note = "Use websockets instead")]
pub fn kill(req: &HttpRequest<ServerState>) -> Result<&'static str> {
    let pid: u32 = req.match_info().get("pid").unwrap_or("").parse()?;
    State::get(|state| {
        state
            .backend_sender
            .send(BackendRequest::KillProcess(pid))
            .map_err(Into::into)
    }).map(|_| "Ok")
}
*/
pub fn status(_req: &HttpRequest<ServerState>) -> impl Responder {
    State::get(|state| serde_json::to_string_pretty(&state).map_err(Into::into))
}

pub fn index(_req: &HttpRequest<ServerState>) -> impl Responder {
    NamedFile::open("ui/index.html")
}

pub fn bundle(_req: &HttpRequest<ServerState>) -> impl Responder {
    NamedFile::open("ui/dist/bundle.js")
}

pub fn bundle_map(_req: &HttpRequest<ServerState>) -> impl Responder {
    NamedFile::open("ui/dist/bundle.js.map")
}
