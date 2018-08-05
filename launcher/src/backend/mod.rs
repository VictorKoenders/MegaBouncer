mod build;

use std::rc::Rc;
pub use self::build::{Project, Build, PostBuildEvent, BuildType, RunType};
use mio_extras::channel::channel;
use state::State;

pub enum BackendRequest {
}

pub fn run() {
    let projects = get_projects();
    let (sender, receiver) = channel();
    State::modify(|s| s.sender = sender);

    loop {

    }
}

fn get_projects() -> Vec<Project> {
    vec![Project {
        directory: "server".to_string(),
        build_process: None,
        running_process: None,
        builds: vec![Rc::new(Build {
            name: "cargo".to_string(),
            directory: "".to_string(),
            pattern: "*.rs,*.toml".to_string(),
            build: BuildType::Cargo,
            after_success: vec![PostBuildEvent::Run(RunType::Cargo)]
        })]
    }]
}
