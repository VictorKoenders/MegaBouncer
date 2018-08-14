use super::{
    Build, ProcessResult, Project, RunType, RunningBuild as BackendRunningBuild,
    RunningProcess as BackendRunningProcess,
};
use mio::{Poll, PollOpt, Ready, Token};
use mio_child_process::{ProcessEvent, StdioChannel};
use state::State;
use std::sync::mpsc::TryRecvError;
use Result;

pub struct Backend {
    pub projects: Vec<Project>,
    pub running_builds: Vec<BackendRunningBuild>,
    pub running_processes: Vec<BackendRunningProcess>,
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

    pub fn kill_process(&mut self, pid: u32) -> Result<()> {
        for process in self
            .running_processes
            .iter_mut()
            .filter(|p| p.process.id() == pid)
        {
            println!("Killing process {}", pid);
            process.process.kill()?;
        }
        Ok(())
    }

    pub fn start_build(&mut self, project_name: String, build_name: String) -> Result<()> {
        for process in self.running_processes
            .iter_mut()
            .filter(|p| p.project_name == project_name && p.build.name == build_name)
        {
            let pid = process.process.id();
            println!("Killing process {} because we're starting a new build", pid);
            process.process.kill()?;
            // State::remove_running_process_by_pid(pid);
        }
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
            let mut running_build =
                BackendRunningBuild::new(project_name.clone(), build.clone(), token)?;
            let id = running_build.process.id();

            if let Err(e) =
                self.poll
                    .register(&running_build.process, token, Ready::all(), PollOpt::edge())
            {
                println!("Killing build {}", running_build.process.id());
                let _ = running_build.process.kill();
                bail!("Could not spawn {}::{}: {:?}", project_name, build.name, e);
            }
            State::add_running_build(project_name.clone(), build.name.clone(), id);
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
            let process = &mut self.running_processes[index];
            let pid = process.process.id();
            println!("Killing process {}", pid);
            process.process.kill()?;
            // State::remove_running_process_by_pid(pid);
        }

        let token = Token(self.next_token);
        let mut running_process =
            BackendRunningProcess::new(project_name.clone(), build.clone(), run.clone(), token)?;
        self.next_token += 1;
        let id = running_process.process.id();
        if let Err(e) = self.poll.register(
            &running_process.process,
            token,
            Ready::all(),
            PollOpt::edge(),
        ) {
            println!("Killing process {} because we could not register it with mio", running_process.process.id());
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
        State::add_running_process(project_name, run, id);

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
                    match e {
                        ProcessEvent::Data(StdioChannel::Stdout, e) => {
                            State::running_build_add_stdout(id, &e);
                        }
                        ProcessEvent::Data(StdioChannel::Stderr, e) => {
                            State::running_build_add_stderr(id, &e);
                        }
                        ProcessEvent::CommandError(e) => {
                            let err = e.into();
                            State::report_error(&err);
                            State::running_build_terminated(id, &err);
                            finished = true;
                        }
                        ProcessEvent::IoError(_, e) => {
                            let err = e.into();
                            State::report_error(&err);
                            State::running_build_terminated(id, &err);
                            finished = true;
                        }
                        ProcessEvent::Utf8Error(_, e) => {
                            let err = e.into();
                            State::report_error(&err);
                            State::running_build_terminated(id, &err);
                            finished = true;
                        }
                        ProcessEvent::Exit(status) => {
                            State::running_build_finished(id, status.code().unwrap_or(0));
                            finished = true;
                            finished_succesfully = status.success();
                        }
                    }
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
                    State::running_build_terminated(id, &format_err!("Channel disconnected"));
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
                    match e {
                        ProcessEvent::Data(StdioChannel::Stdout, e) => {
                            State::running_process_add_stdout(id, &e);
                        }
                        ProcessEvent::Data(StdioChannel::Stderr, e) => {
                            State::running_process_add_stderr(id, &e);
                        }
                        ProcessEvent::CommandError(e) | ProcessEvent::IoError(_, e) => {
                            let err = e.into();
                            State::running_process_terminated(id, &err);
                            finished = true;
                        }
                        ProcessEvent::Utf8Error(_, e) => {
                            let err = e.into();
                            State::running_process_terminated(id, &err);
                            finished = true;
                        }
                        ProcessEvent::Exit(status) => {
                            State::running_process_finished(id, status.code().unwrap_or(0));
                            finished = true;
                            finished_succesfully = status.success();
                        }
                    }
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
                    State::running_process_terminated(id, &format_err!("Channel disconnected"));
                    return ProcessResult::failed();
                }
            }
        }
    }
}
