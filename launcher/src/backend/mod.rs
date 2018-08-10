mod backend_impl;
mod build;
mod running_build;

pub use self::backend_impl::Backend;
pub use self::build::{Build, BuildType, PostBuildEvent, Project, RunType};
pub use self::running_build::{RunningBuild, RunningProcess};

use mio::{Events, PollOpt, Ready, Token};
use mio_extras::channel::channel;
use state::State;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use Result;

#[derive(Debug)]
pub enum BackendRequest {
    StartBuild {
        project_name: String,
        build_name: String,
    },
    KillProcess(u32),
}

pub fn run(base_dir: &str) -> Result<()> {
    let projects = get_projects(base_dir)?;
    State::modify(|state| {
        state.projects = projects.clone();
    });
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
                        BackendRequest::StartBuild {
                            project_name,
                            build_name,
                        } => {
                            if let Err(e) = backend.start_build(project_name, build_name) {
                                State::report_error(e);
                            }
                        },
                        BackendRequest::KillProcess(pid) => {
                            if let Err(e) = backend.kill_process(pid) {
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
                                PostBuildEvent::Run(run_type) => {
                                    let project_name =
                                        backend.running_builds[index].project_name.clone();
                                    let build = backend.running_builds[index].build.clone();
                                    start_process = Some((project_name, build, run_type.clone()));
                                }
                                PostBuildEvent::TriggerBuild { name } => {
                                    let project_name =
                                        backend.running_builds[index].project_name.clone();
                                    start_build = Some((project_name, name.clone()));
                                }
                            }
                        }
                        if let Some((project_name, build_name)) = start_build {
                            backend.start_build(project_name, build_name)?;
                        }
                        if let Some((project_name, build, run_type)) = start_process {
                            backend.start_process(project_name, build, run_type)?;
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

fn get_projects_in_dir(dir: &PathBuf) -> Result<Option<Project>> {
    let mut path_buf = dir.to_path_buf();
    let project_name = match dir.file_name().and_then(|s| s.to_str()) {
        Some(f) => f,
        None => return Ok(None),
    };
    let mut result = Project {
        name: project_name.to_string(),
        builds: Vec::new(),
    };
    path_buf.push("Cargo.toml");
    if path_buf.exists() && path_buf.is_file() {
        path_buf.pop();
        result.builds.push(Build {
            name: String::from("cargo"),
            directory: path_buf.to_string_lossy().to_string(),
            pattern: String::from(".rs,.toml"),
            build: BuildType::Cargo,
            after_success: Some(PostBuildEvent::Run(RunType::Cargo)),
        });
    } else {
        path_buf.pop();
    }

    path_buf.push("ui");
    if path_buf.exists() {
        result.builds.push(Build {
            name: String::from("webpack"),
            directory: path_buf.to_string_lossy().to_string(),
            pattern: String::from(".ts,.tsx"),
            build: BuildType::TypescriptReactWebpack,
            after_success: if project_name != "launcher" {
                Some(PostBuildEvent::TriggerBuild {
                    name: String::from("cargo"),
                })
            } else {
                None
            },
        });
    }

    if result.builds.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

pub fn get_projects(base_dir: &str) -> Result<Vec<Project>> {
    let mut result = Vec::new();
    for dir in fs::read_dir(base_dir)?.filter_map(|d| d.ok()) {
        if !dir.path().is_dir() {
            continue;
        }
        let name = dir.file_name();
        let name = match name.to_str() {
            Some(n) => n,
            None => continue,
        };
        if name.starts_with(".") {
            continue;
        }
        result.extend(get_projects_in_dir(&dir.path())?);
    }
    if result.is_empty() {
        Err(format_err!("No projects found in directory"))
    } else {
        Ok(result)
    }
}
