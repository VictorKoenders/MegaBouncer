use std::sync::Mutex;
use mio_extras::channel::{Sender, channel};
use backend::{BackendRequest, RunType};
use std::fmt;
use chrono::{Utc, DateTime};
use failure::Error;
use std::process::ExitStatus;

pub struct State {
    pub running_builds: Vec<StateBuildProcess>,
    pub running_processes: Vec<StateProcess>,
    pub finished_builds: Vec<StateBuildProcess>,
    pub sender: Sender<BackendRequest>,
    pub errors: Vec<(DateTime<Utc>, Error)>,
}

impl fmt::Debug for State {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("State")
            .field("running_builds", &self.running_builds)
            .field("finished_builds", &self.finished_builds)
            .field("running_processes", &self.running_processes)
            .field("errors", &self.errors)
            .finish()
    }
}

lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(State {
        running_builds: Vec::new(),
        finished_builds: Vec::new(),
        running_processes: Vec::new(),
        sender: channel().0,
        errors: Vec::new(),
    });
}

impl State {
    pub fn report_error(e: Error) {
        State::modify(|state| {
            state.errors.push((Utc::now(), e)); 
        });
    }
    pub fn get<F>(cb: F)
        where F : FnOnce(&State) {
        let state = STATE.lock().unwrap();
        cb(&state);
    }

    pub fn modify<F>(cb: F) 
        where F : FnOnce(&mut State) {
        let mut state = STATE.lock().unwrap();
        cb(&mut state);
    }
}

#[derive(Debug, Clone)]
pub struct StateProcess {
    pub directory: String,
    pub run_type: RunType,
    pub id: u32,
    pub status: StateProcessState,
    pub stdout: String,
    pub stderr: String,
}

impl StateProcess {
    pub fn new(directory: String, run_type: RunType, id: u32) -> StateProcess {
        StateProcess {
            directory,
            run_type,
            id,
            status: StateProcessState::Running,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}


#[derive(Debug, Clone)]
pub struct StateBuildProcess {
    pub directory: String,
    pub build: String,
    pub id: u32,
    pub status: StateProcessState,
    pub stdout: String,
    pub stderr: String,
}

impl StateBuildProcess {
    pub fn new(directory: String, build: String, id: u32) -> StateBuildProcess {
        StateBuildProcess {
            directory,
            build,
            id,
            status: StateProcessState::Running,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StateProcessState {
    Running,
    Failed,
    Success(ExitStatus),
}
