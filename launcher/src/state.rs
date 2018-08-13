use backend::{BackendRequest, Project, RunType};
use chrono::{DateTime, Utc};
use failure::Error;
use mio_extras::channel::{channel as mio_channel, Sender as MioSender};
use std::fmt;
use std::sync::mpsc::{channel as std_channel, Sender as StdSender};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Serialize)]
pub struct State {
    pub running_processes: Vec<RunningProcess>,
    pub running_builds: Vec<RunningBuild>,
    pub finished_builds: Vec<FinishedBuild>,
    #[serde(skip_serializing)]
    pub backend_sender: MioSender<BackendRequest>,
    #[serde(skip_serializing)]
    pub change_sender: StdSender<StateChange>,
    pub projects: Vec<Project>,
    pub errors: Vec<StateError>,
}

#[derive(Debug)]
pub enum StateChange {
    ErrorAdded(StateError),

    ProjectsSet(Vec<Project>),

    RunningProcessAdded(RunningProcess),
    RunningProcessRemoved(Uuid),
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
        backend_sender: mio_channel().0,
        change_sender: std_channel().0,
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
        self.change_sender
            .send(StateChange::ErrorAdded(error))
            .expect("Change sender failed");
    }
    pub fn report_error(e: &Error) {
        let mut state = STATE.lock().unwrap();
        state.report_error_internal(&e);
    }
    pub fn get<F, T>(cb: F) -> Result<T, Error>
    where
        F: FnOnce(&State) -> Result<T, Error>,
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
        state
            .change_sender
            .send(StateChange::ProjectsSet(p))
            .expect("Change sender failed");
    }

    pub fn set_backend_sender(sender: MioSender<BackendRequest>) {
        let mut state = STATE.lock().unwrap();
        state.backend_sender = sender;
    }

    pub fn set_change_sender(sender: StdSender<StateChange>) {
        let mut state = STATE.lock().unwrap();
        state.change_sender = sender;
    }
}
impl State {
    pub fn remove_running_process_by_pid(pid: u32) {
        let mut state = STATE.lock().unwrap();
        while let Some(index) = state.running_processes.iter().position(|p| p.pid == pid) {
            let process = state.running_processes.remove(index);
            state
                .change_sender
                .send(StateChange::RunningProcessRemoved(process.uuid))
                .expect("Change sender failed");
        }
    }

    pub fn add_running_process(project_name: String, run_type: RunType, id: u32) {
        let mut state = STATE.lock().unwrap();
        let process = RunningProcess::new(project_name, run_type, id);
        state.running_processes.push(process.clone());
        state
            .change_sender
            .send(StateChange::RunningProcessAdded(process))
            .expect("Change sender failed");
    }

    pub fn running_process_add_stdout(pid: u32, addition: &str) {
        let mut state = STATE.lock().unwrap();
        let mut msg = None;
        if let Some(running_process) = state.running_processes.iter_mut().find(|b| b.pid == pid) {
            running_process.stdout += addition;
            msg = Some(StateChange::RunningProcessStdout(
                running_process.uuid.clone(),
                addition.to_string(),
            ));
        }
        if let Some(msg) = msg {
            state.change_sender.send(msg).expect("Change sender failed");
        }
    }

    pub fn running_process_add_stderr(pid: u32, addition: &str) {
        let mut state = STATE.lock().unwrap();
        let mut msg = None;
        if let Some(running_process) = state.running_processes.iter_mut().find(|b| b.pid == pid) {
            running_process.stderr += addition;
            msg = Some(StateChange::RunningProcessStderr(
                running_process.uuid.clone(),
                addition.to_string(),
            ));
        }
        if let Some(msg) = msg {
            state.change_sender.send(msg).expect("Change sender failed");
        }
    }

    pub fn running_process_terminated(pid: u32, err: &Error) {
        let mut state = STATE.lock().unwrap();
        if let Some(index) = state.running_processes.iter().position(|b| b.pid == pid) {
            let process = state.running_processes.remove(index);
            let err = format!("{}", err);
            state
                .change_sender
                .send(StateChange::RunningProcessTerminated(process.uuid, err))
                .expect("Change sender failed");
        }
    }

    pub fn running_process_finished(pid: u32, status: i32) {
        let mut state = STATE.lock().unwrap();
        if let Some(index) = state.running_processes.iter().position(|b| b.pid == pid) {
            let mut process = state.running_processes.remove(index);
            state
                .change_sender
                .send(StateChange::RunningProcessFinished(process.uuid, status))
                .expect("Change sender failed");
        }
    }
}
impl State {
    pub fn add_running_build(project_name: String, build_name: String, id: u32) {
        let mut state = STATE.lock().unwrap();
        let build = RunningBuild::new(project_name, build_name, id);

        state.running_builds.push(build.clone());
        state
            .change_sender
            .send(StateChange::RunningBuildAdded(build))
            .expect("Change sender failed");
    }

    pub fn running_build_add_stdout(pid: u32, addition: &str) {
        let mut state = STATE.lock().unwrap();
        let mut msg = None;
        if let Some(running_build) = state.running_builds.iter_mut().find(|b| b.pid == pid) {
            running_build.stdout += addition;
            msg = Some(StateChange::RunningBuildStdout(
                running_build.uuid.clone(),
                addition.to_string(),
            ));
        }
        if let Some(msg) = msg {
            state.change_sender.send(msg).expect("Change sender failed");
        }
    }

    pub fn running_build_add_stderr(pid: u32, addition: &str) {
        let mut state = STATE.lock().unwrap();
        let mut msg = None;
        if let Some(running_build) = state.running_builds.iter_mut().find(|b| b.pid == pid) {
            running_build.stderr += addition;
            msg = Some(StateChange::RunningBuildStderr(
                running_build.uuid.clone(),
                addition.to_string(),
            ));
        }
        if let Some(msg) = msg {
            state.change_sender.send(msg).expect("Change sender failed");
        }
    }

    pub fn running_build_terminated(pid: u32, err: &Error) {
        let mut state = STATE.lock().unwrap();
        if let Some(index) = state.running_builds.iter().position(|b| b.pid == pid) {
            let build = state.running_builds.remove(index);
            let mut finished_build: FinishedBuild = build.into();
            let err = format!("{}", err);
            finished_build.error = Some(err.clone());
            state
                .change_sender
                .send(StateChange::RunningBuildTerminated(
                    finished_build.uuid.clone(),
                    err,
                )).expect("Change sender failed");
            state.finished_builds.insert(0, finished_build);
        }
    }

    pub fn running_build_finished(pid: u32, status: i32) {
        let mut state = STATE.lock().unwrap();
        if let Some(index) = state.running_builds.iter().position(|b| b.pid == pid) {
            let mut process: FinishedBuild = state.running_builds.remove(index).into();
            process.status = status;
            state
                .change_sender
                .send(StateChange::RunningBuildFinished(
                    process.uuid.clone(),
                    status,
                )).expect("Change sender failed");
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
