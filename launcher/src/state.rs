use actix::Recipient;
use backend::{BackendRequest, Project, RunType};
use chrono::{DateTime, Utc};
use failure::Error;
use mio_extras::channel::{channel, Sender};
use std::fmt;
use std::sync::Mutex;
use uuid::Uuid;
use Result;

#[derive(Serialize)]
pub struct State {
    pub running_processes: Vec<RunningProcess>,
    pub running_builds: Vec<RunningBuild>,
    pub finished_processes: Vec<FinishedProcess>,
    pub finished_builds: Vec<FinishedBuild>,
    #[serde(skip_serializing)]
    pub backend_sender: Sender<BackendRequest>,
    #[serde(skip_serializing)]
    pub change_sender: Option<Recipient<StateChange>>,
    pub projects: Vec<Project>,
    pub errors: Vec<StateError>,
}

#[derive(Debug, Message, Serialize)]
pub enum StateChange {
    ErrorAdded(StateError),

    ProjectsSet(Vec<Project>),

    RunningProcessAdded(RunningProcess),
    RunningProcessStdout(Uuid, String),
    RunningProcessStderr(Uuid, String),
    RunningProcessTerminated(Uuid, String),
    RunningProcessFinished(Uuid, i32),

    RunningBuildAdded(RunningBuild),
    RunningBuildStdout(Uuid, String),
    RunningBuildStderr(Uuid, String),
    RunningBuildTerminated(Uuid, String),
    RunningBuildFinished(Uuid, i32),
}

// TODO: We need to make our own Error type that's cloneable, so we can send this error to several places within the application
// For now we're just using `format!("{}", e)`
// https://github.com/rust-lang/rust/issues/24135
// https://github.com/rust-lang-nursery/failure/issues/148
#[derive(Clone, Debug, Serialize)]
pub struct StateError {
    pub time: DateTime<Utc>,
    pub error: String,
}

impl fmt::Debug for State {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("State")
            .field("running_builds", &self.running_builds)
            .field("finished_builds", &self.finished_builds)
            .field("running_processes", &self.running_processes)
            .field("finished_processes", &self.finished_processes)
            .field("projects", &self.projects)
            .field("errors", &self.errors)
            .finish()
    }
}

lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(State {
        running_builds: Vec::new(),
        finished_builds: Vec::new(),
        running_processes: Vec::new(),
        finished_processes: Vec::new(),
        backend_sender: channel().0,
        change_sender: None,
        projects: Vec::new(),
        errors: Vec::new(),
    });
}

impl State {
    fn report_error_internal(&mut self, e: &Error) {
        let error = StateError {
            time: Utc::now(),
            error: format!("{}", e),
        };
        self.errors.push(error.clone());
        self.emit_change(StateChange::ErrorAdded(error));
    }
    pub fn report_error(e: &Error) {
        let mut state = STATE.lock().unwrap();
        state.report_error_internal(&e);
    }
    pub fn get<F, T>(cb: F) -> Result<T>
    where
        F: FnOnce(&State) -> Result<T>,
    {
        let mut state = STATE.lock().unwrap();
        let result = cb(&state);
        if let Err(e) = &result {
            state.report_error_internal(e);
        }
        result
    }

    pub fn set_projects(p: Vec<Project>) {
        let mut state = STATE.lock().unwrap();
        state.projects = p.clone();
        state.emit_change(StateChange::ProjectsSet(p));
    }

    pub fn set_backend_sender(sender: Sender<BackendRequest>) {
        let mut state = STATE.lock().unwrap();
        state.backend_sender = sender;
    }

    pub fn set_change_sender(sender: Recipient<StateChange>) {
        let mut state = STATE.lock().unwrap();
        state.change_sender = Some(sender);
    }

    fn emit_change(&mut self, change: StateChange) {
        if let Some(change_sender) = &self.change_sender {
            change_sender
                .do_send(change)
                .expect("Could not send change to change_sender");
        }
    }
}
impl State {
    pub fn add_running_process(project_name: String, run_type: RunType, id: u32) {
        let mut state = STATE.lock().unwrap();
        let process = RunningProcess::new(project_name, run_type, id);
        state.running_processes.push(process.clone());
        state.emit_change(StateChange::RunningProcessAdded(process));
    }

    pub fn running_process_add_stdout(pid: u32, addition: &str) {
        let mut state = STATE.lock().unwrap();
        let mut msg = None;
        if let Some(running_process) = state.running_processes.iter_mut().find(|b| b.pid == pid) {
            running_process.stdout += addition;
            msg = Some(StateChange::RunningProcessStdout(
                running_process.uuid,
                addition.to_string(),
            ));
        }
        if let Some(msg) = msg {
            state.emit_change(msg);
        }
    }

    pub fn running_process_add_stderr(pid: u32, addition: &str) {
        let mut state = STATE.lock().unwrap();
        let mut msg = None;
        if let Some(running_process) = state.running_processes.iter_mut().find(|b| b.pid == pid) {
            running_process.stderr += addition;
            msg = Some(StateChange::RunningProcessStderr(
                running_process.uuid,
                addition.to_string(),
            ));
        }
        if let Some(msg) = msg {
            state.emit_change(msg);
        }
    }

    pub fn running_process_terminated(pid: u32, err: &Error) {
        let mut state = STATE.lock().unwrap();
        if let Some(index) = state.running_processes.iter().position(|b| b.pid == pid) {
            let process = state.running_processes.remove(index);
            let err = format!("{}", err);
            let mut process: FinishedProcess = process.into();
            process.error = Some(err.clone());
            state.emit_change(StateChange::RunningProcessTerminated(process.uuid, err));
            state.finished_processes.insert(0, process);
        }
    }

    pub fn running_process_finished(pid: u32, status: i32) {
        let mut state = STATE.lock().unwrap();
        if let Some(index) = state.running_processes.iter().position(|b| b.pid == pid) {
            let mut process = state.running_processes.remove(index);
            let mut process: FinishedProcess = process.into();
            process.status = status;
            state.emit_change(StateChange::RunningProcessFinished(process.uuid, status));
            state.finished_processes.insert(0, process);
        }
    }
}
impl State {
    pub fn add_running_build(project_name: String, build_name: String, id: u32) {
        let mut state = STATE.lock().unwrap();
        let build = RunningBuild::new(project_name, build_name, id);

        state.running_builds.push(build.clone());
        state.emit_change(StateChange::RunningBuildAdded(build));
    }

    pub fn running_build_add_stdout(pid: u32, addition: &str) {
        let mut state = STATE.lock().unwrap();
        let mut msg = None;
        if let Some(running_build) = state.running_builds.iter_mut().find(|b| b.pid == pid) {
            running_build.stdout += addition;
            msg = Some(StateChange::RunningBuildStdout(
                running_build.uuid,
                addition.to_string(),
            ));
        }
        if let Some(msg) = msg {
            state.emit_change(msg);
        }
    }

    pub fn running_build_add_stderr(pid: u32, addition: &str) {
        let mut state = STATE.lock().unwrap();
        let mut msg = None;
        if let Some(running_build) = state.running_builds.iter_mut().find(|b| b.pid == pid) {
            running_build.stderr += addition;
            msg = Some(StateChange::RunningBuildStderr(
                running_build.uuid,
                addition.to_string(),
            ));
        }
        if let Some(msg) = msg {
            state.emit_change(msg);
        }
    }

    pub fn running_build_terminated(pid: u32, err: &Error) {
        let mut state = STATE.lock().unwrap();
        if let Some(index) = state.running_builds.iter().position(|b| b.pid == pid) {
            let build = state.running_builds.remove(index);
            let mut finished_build: FinishedBuild = build.into();
            let err = format!("{}", err);
            finished_build.error = Some(err.clone());
            state.emit_change(StateChange::RunningBuildTerminated(
                finished_build.uuid,
                err,
            ));
            state.finished_builds.insert(0, finished_build);
        }
    }

    pub fn running_build_finished(pid: u32, status: i32) {
        let mut state = STATE.lock().unwrap();
        if let Some(index) = state.running_builds.iter().position(|b| b.pid == pid) {
            let mut process: FinishedBuild = state.running_builds.remove(index).into();
            process.status = status;
            state.emit_change(StateChange::RunningBuildFinished(process.uuid, status));
            state.finished_builds.insert(0, process);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RunningProcess {
    pub uuid: Uuid,
    pub directory: String,
    pub run_type: RunType,
    pub pid: u32,
    pub stdout: String,
    pub stderr: String,
}

impl RunningProcess {
    pub fn new(directory: String, run_type: RunType, pid: u32) -> RunningProcess {
        RunningProcess {
            uuid: Uuid::new_v4(),
            directory,
            run_type,
            pid,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FinishedProcess {
    pub uuid: Uuid,
    pub directory: String,
    pub run_type: RunType,
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
    pub error: Option<String>,
}

impl From<RunningProcess> for FinishedProcess {
    fn from(p: RunningProcess) -> FinishedProcess {
        FinishedProcess {
            uuid: p.uuid,
            directory: p.directory,
            run_type: p.run_type,
            stdout: p.stdout,
            stderr: p.stderr,
            status: 0,
            error: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RunningBuild {
    pub uuid: Uuid,
    pub directory: String,
    pub build: String,
    pub started_on: DateTime<Utc>,
    pub pid: u32,
    pub stdout: String,
    pub stderr: String,
}

impl RunningBuild {
    pub fn new(directory: String, build: String, pid: u32) -> RunningBuild {
        RunningBuild {
            uuid: Uuid::new_v4(),
            directory,
            build,
            started_on: Utc::now(),
            pid,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FinishedBuild {
    pub uuid: Uuid,
    pub directory: String,
    pub build: String,
    pub started_on: DateTime<Utc>,
    pub ended_on: DateTime<Utc>,
    pub status: i32,
    pub error: Option<String>,
    pub pid: u32,
    pub stdout: String,
    pub stderr: String,
}

impl From<RunningBuild> for FinishedBuild {
    fn from(build: RunningBuild) -> FinishedBuild {
        FinishedBuild {
            uuid: build.uuid,
            directory: build.directory,
            build: build.build,
            started_on: build.started_on,
            ended_on: Utc::now(),
            status: 0,
            error: None,
            pid: build.pid,
            stdout: build.stdout,
            stderr: build.stderr,
        }
    }
}
