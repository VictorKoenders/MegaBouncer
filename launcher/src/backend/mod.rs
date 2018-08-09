mod backend_impl;
mod build;
mod running_build;

pub use self::backend_impl::Backend;
pub use self::build::{Build, BuildType, PostBuildEvent, Project, RunType};
pub use self::running_build::{RunningBuild, RunningProcess};

use mio::{Events, PollOpt, Ready, Token};
use mio_extras::channel::channel;
use state::State;
use std::sync::mpsc::TryRecvError;
use Result;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum BackendRequest {
    StartBuild { directory: String, name: String },
}

pub fn run(base_dir: &str) -> Result<()> {
    let projects = get_projects();
    let mut backend = Backend::new(projects)?;

    let (sender, receiver) = channel();
    State::modify(|s| s.sender = sender);
    let mut events = Events::with_capacity(10);
    let receiver_token = Token(1);
    backend
        .poll
        .register(&receiver, receiver_token, Ready::all(), PollOpt::edge())?;

    loop {
        if let Err(e) = backend.poll.poll(&mut events, None) {
            bail!("Poll error: {:?}", e);
        }
        for ev in &events {
            if ev.token() == receiver_token {
                'recv_loop: loop {
                    let request = match receiver.try_recv() {
                        Ok(r) => r,
                        Err(TryRecvError::Empty) => break 'recv_loop,
                        Err(e) => {
                            bail!("Receiver error: {:?}", e);
                        }
                    };
                    match request {
                        BackendRequest::StartBuild { directory, name } => {
                            if let Err(e) = backend.start_build(base_dir, directory, name) {
                                State::report_error(e);
                            }
                        }
                    }
                }
            } else if let Some(index) = backend
                .running_builds
                .iter()
                .position(|b| b.token == ev.token())
            {
                let result = backend.update_running_build(index);

                if result.finished {
                    if result.finished_succesfully {
                        let mut start_build = None;
                        let mut start_process = None;
                        if let Some(follow_up) = &backend.running_builds[index].build.after_success
                        {
                            match follow_up {
                                PostBuildEvent::Run(_type) => {
                                    let directory = backend.running_builds[index].directory.clone();
                                    start_process = Some((directory, _type.clone()));
                                }
                                PostBuildEvent::TriggerBuild { name } => {
                                    let directory = backend.running_builds[index].directory.clone();
                                    start_build = Some((directory, name.clone()));
                                }
                            }
                        }
                        if let Some((directory, build)) = start_build {
                            backend.start_build(base_dir, directory, build)?;
                        }
                        if let Some((directory, run_type)) = start_process {
                            backend.start_process(base_dir, directory, run_type)?;
                        }
                    }
                    backend.running_builds.remove(index);
                }
            } else if let Some(index) = backend
                .running_processes
                .iter()
                .position(|b| b.token == ev.token())
            {
                backend.update_running_process(index);
            }
        }
    }
}

pub struct ProcessResult {
    pub finished: bool,
    pub finished_succesfully: bool,
}

impl ProcessResult {
    pub fn not_finished() -> ProcessResult {
        ProcessResult {
            finished: false,
            finished_succesfully: false,
        }
    }
    pub fn failed() -> ProcessResult {
        ProcessResult {
            finished: true,
            finished_succesfully: false,
        }
    }
}

fn get_projects() -> Vec<Project> {
    vec![Project {
        directory: "server".to_string(),
        builds: vec![Build {
            name: "cargo".to_string(),
            directory: "".to_string(),
            pattern: "*.rs,*.toml".to_string(),
            build: BuildType::Cargo,
            after_success: Some(PostBuildEvent::Run(RunType::Cargo)),
        }],
    }]
}
