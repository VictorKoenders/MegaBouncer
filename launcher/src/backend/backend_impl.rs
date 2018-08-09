use super::{Build, ProcessResult, Project, RunType, RunningBuild, RunningProcess};
use chrono::Utc;
use mio::{Poll, PollOpt, Ready, Token};
use mio_child_process::{ProcessEvent, StdioChannel};
use state::{State, StateBuildProcess, StateError, StateProcess, StateProcessState};
use std::sync::mpsc::TryRecvError;
use Result;

pub struct Backend {
    pub projects: Vec<Project>,
    pub running_builds: Vec<RunningBuild>,
    pub running_processes: Vec<RunningProcess>,
    pub poll: Poll,
    next_token: usize,
}

impl Backend {
    pub fn new(projects: Vec<Project>) -> Result<Backend> {
        let poll = Poll::new()?;
        Ok(Backend {
            projects,
            running_builds: Vec::new(),
            running_processes: Vec::new(),
            next_token: 10,
            poll,
        })
    }

    pub fn start_build(&mut self, project_name: String, build_name: String) -> Result<()> {
        for b in self
            .running_builds
            .iter()
            .filter(|b| b.project_name == project_name)
        {
            if b.build.name == build_name {
                bail!(
                    "Could not start {}::{}, already running",
                    project_name,
                    build_name
                );
            }
        }
        let build = {
            let project = match self.projects.iter().find(|p| p.name == project_name) {
                Some(p) => p,
                None => {
                    bail!("Could not start {:?}, not found", project_name);
                }
            };
            let build = match project.builds.iter().find(|b| b.name == build_name) {
                Some(b) => b,
                None => {
                    bail!(
                        "Could not start {}::{}, available:{:?}",
                        project_name,
                        build_name,
                        project.builds
                    );
                }
            };

            let token = Token(self.next_token);
            self.next_token += 1;
            let mut running_build = RunningBuild::new(project_name.clone(), build.clone(), token)?;
            let id = running_build.process.id();

            if let Err(e) =
                self.poll
                    .register(&running_build.process, token, Ready::all(), PollOpt::edge())
            {
                let _ = running_build.process.kill();
                bail!("Could not spawn {}::{}: {:?}", project_name, build.name, e);
            }
            State::modify(|s| {
                s.running_builds.push(StateBuildProcess::new(
                    project_name.clone(),
                    build.name.clone(),
                    id,
                ));
            });
            running_build
        };
        self.running_builds.push(build);
        Ok(())
    }

    pub fn start_process(
        &mut self,
        project_name: String,
        build: Build,
        run: RunType,
    ) -> Result<()> {
        println!("Starting {}::{} {:?}", project_name, build.name, run);
        while let Some(index) = self.running_processes.iter().position(|p| {
            p.project_name == project_name && p.build.name == build.name && p.run_type == run
        }) {
            let mut process = self.running_processes.remove(index);
            let id = process.process.id();
            println!("Killing pid {}", id);
            process.process.kill()?;
            State::modify(|state| {
                state.running_processes.retain(|p| p.id != id);
            });
        }

        let token = Token(self.next_token);
        let mut running_process =
            RunningProcess::new(project_name.clone(), build.clone(), run.clone(), token)?;
        self.next_token += 1;
        let id = running_process.process.id();
        if let Err(e) = self.poll.register(
            &running_process.process,
            token,
            Ready::all(),
            PollOpt::edge(),
        ) {
            let _ = running_process.process.kill();
            bail!(
                "Could not spawn {}::{} {:?}: {:?}",
                project_name,
                build.name,
                run,
                e
            );
        }
        self.running_processes.push(running_process);
        State::modify(|s| {
            s.running_processes
                .push(StateProcess::new(project_name, run, id));
        });

        Ok(())
    }

    pub fn update_running_build(&mut self, index: usize) -> ProcessResult {
        let mut finished = false;
        let mut finished_succesfully = false;
        loop {
            let result = self.running_builds[index].process.try_recv();
            match result {
                Ok(e) => {
                    let id = self.running_builds[index].process.id();
                    State::modify(|state| {
                        let index = state
                            .running_builds
                            .iter()
                            .position(|p| p.id == id)
                            .unwrap();
                        match e {
                            ProcessEvent::Data(StdioChannel::Stdout, e) => {
                                state.running_builds[index].stdout += &e
                            }
                            ProcessEvent::Data(StdioChannel::Stderr, e) => {
                                state.running_builds[index].stderr += &e
                            }
                            ProcessEvent::CommandError(e) => {
                                state.errors.push(StateError {
                                    time: Utc::now(),
                                    error: e.into(),
                                });
                                let mut process = state.running_builds.remove(index);
                                process.status = StateProcessState::Failed;
                                state.finished_builds.push(process);
                                finished = true;
                            }
                            ProcessEvent::IoError(_, e) => {
                                state.errors.push(StateError {
                                    time: Utc::now(),
                                    error: e.into(),
                                });
                                let mut process = state.running_builds.remove(index);
                                process.status = StateProcessState::Failed;
                                state.finished_builds.push(process);
                                finished = true;
                            }
                            ProcessEvent::Utf8Error(_, e) => {
                                state.errors.push(StateError {
                                    time: Utc::now(),
                                    error: e.into(),
                                });
                                let mut process = state.running_builds.remove(index);
                                process.status = StateProcessState::Failed;
                                state.finished_builds.push(process);
                                finished = true;
                            }
                            ProcessEvent::Exit(status) => {
                                let mut process = state.running_builds.remove(index);
                                process.status =
                                    StateProcessState::Success(status.code().unwrap_or(0));
                                state.finished_builds.push(process);
                                finished = true;
                                finished_succesfully = status.success();
                            }
                        }
                    });
                    if finished {
                        return ProcessResult {
                            finished,
                            finished_succesfully,
                        };
                    }
                }
                Err(TryRecvError::Empty) => return ProcessResult::not_finished(),
                Err(TryRecvError::Disconnected) => {
                    let id = self.running_builds[index].process.id();
                    State::modify(|state| {
                        if let Some(index) = state.running_builds.iter().position(|p| p.id == id) {
                            let mut process = state.running_builds.remove(index);
                            process.status = StateProcessState::Failed;
                            state.finished_builds.push(process);
                        }
                    });
                    return ProcessResult::failed();
                }
            }
        }
    }

    pub fn update_running_process(&mut self, index: usize) -> ProcessResult {
        let mut finished = false;
        let mut finished_succesfully = false;
        loop {
            let result = self.running_processes[index].process.try_recv();
            match result {
                Ok(e) => {
                    let id = self.running_processes[index].process.id();
                    State::modify(|state| {
                        let index = state
                            .running_processes
                            .iter()
                            .position(|p| p.id == id)
                            .unwrap();
                        match e {
                            ProcessEvent::Data(StdioChannel::Stdout, e) => {
                                state.running_processes[index].stdout += &e
                            }
                            ProcessEvent::Data(StdioChannel::Stderr, e) => {
                                state.running_processes[index].stderr += &e
                            }
                            ProcessEvent::CommandError(e) => {
                                state.errors.push(StateError {
                                    time: Utc::now(),
                                    error: e.into(),
                                });
                                state.running_processes.remove(index);
                                finished = true;
                            }
                            ProcessEvent::IoError(_, e) => {
                                state.errors.push(StateError {
                                    time: Utc::now(),
                                    error: e.into(),
                                });
                                state.running_processes.remove(index);
                                finished = true;
                            }
                            ProcessEvent::Utf8Error(_, e) => {
                                state.errors.push(StateError {
                                    time: Utc::now(),
                                    error: e.into(),
                                });
                                state.running_processes.remove(index);
                                finished = true;
                            }
                            ProcessEvent::Exit(status) => {
                                state.running_processes.remove(index);
                                finished = true;
                                finished_succesfully = status.success();
                            }
                        }
                    });
                    if finished {
                        return ProcessResult {
                            finished,
                            finished_succesfully,
                        };
                    }
                }
                Err(TryRecvError::Empty) => return ProcessResult::not_finished(),
                Err(TryRecvError::Disconnected) => {
                    let id = self.running_builds[index].process.id();
                    State::modify(|state| {
                        if let Some(index) = state.running_builds.iter().position(|p| p.id == id) {
                            let mut process = state.running_builds.remove(index);
                            process.status = StateProcessState::Failed;
                            state.finished_builds.push(process);
                        }
                    });
                    return ProcessResult::failed();
                }
            }
        }
    }
}
