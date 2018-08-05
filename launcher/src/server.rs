use actix_web::{server, App, HttpRequest, Responder};
use state::State;

fn status(_req: &HttpRequest) -> impl Responder {
    let mut result = String::new();
    State::get(|state| {
        result += "Modules:\n";
        for module in &state.modules {
            result += &format!("{}\n", module.name);
        }
        result += "\nProcesses\n";
        for process in &state.processes {
            result += &format!("{}: {}", process.status, process.module.name);
        }
    });
    result
}

pub fn run() {
    server::new(|| {
        App::new()
            .resource("/", |r| r.f(status))
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run();
}