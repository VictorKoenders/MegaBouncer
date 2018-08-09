use backend::{BackendRequest, Project, RunType};
use chrono::{DateTime, Utc};
use failure::Error;
use mio_extras::channel::{channel, Sender};
use std::fmt;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Serialize)]
pub struct State {
    pub running_processes: Vec<RunningProcess>,
    pub running_builds: Vec<RunningBuild>,
    pub finished_builds: Vec<FinishedBuild>,
    #[serde(skip_serializing)]
    pub sender: Sender<BackendRequest>,
    pub projects: Vec<Project>,
    pub errors: Vec<StateError>,
}

#[derive(Debug, Serialize)]
pub struct StateError {
    pub time: DateTime<Utc>,
    #[serde(serialize_with = "error_to_string")]
    pub error: Error,
}

fn error_to_string<S>(err: &Error, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ::serde::Serializer,
{
    serializer.serialize_str(&format!("{:?}", err))
}

fn optional_error_to_string<S>(err: &Option<Error>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ::serde::Serializer,
{
    serializer.serialize_str(&format!("{:?}", err))
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
        sender: channel().0,
        projects: Vec::new(),
        errors: Vec::new(),
    });
}

impl State {
    pub fn report_error(e: Error) {
        State::modify(|state| {
            state.errors.push(StateError {
                time: Utc::now(),
                error: e,
            });
        });
    }
    pub fn get<F>(cb: F)
    where
        F: FnOnce(&State),
    {
        let state = STATE.lock().unwrap();
        cb(&state);
    }

    pub fn modify<F>(cb: F)
    where
        F: FnOnce(&mut State),
    {
        let mut state = STATE.lock().unwrap();
        cb(&mut state);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RunningProcess {
    pub uuid: Uuid,
    pub directory: String,
    pub run_type: RunType,
    pub id: u32,
    pub stdout: String,
    pub stderr: String,
}

impl RunningProcess {
    pub fn new(directory: String, run_type: RunType, id: u32) -> RunningProcess {
        RunningProcess {
            uuid: Uuid::new_v4(),
            directory,
            run_type,
            id,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RunningBuild {
    pub uuid: Uuid,
    pub directory: String,
    pub build: String,
    pub started_on: DateTime<Utc>,
    pub id: u32,
    pub stdout: String,
    pub stderr: String,
}

impl RunningBuild {
    pub fn new(directory: String, build: String, id: u32) -> RunningBuild {
        RunningBuild {
            uuid: Uuid::new_v4(),
            directory,
            build,
            started_on: Utc::now(),
            id,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FinishedBuild {
    pub uuid: Uuid,
    pub directory: String,
    pub build: String,
    pub started_on: DateTime<Utc>,
    pub ended_on: DateTime<Utc>,
    pub status: i32,
    #[serde(serialize_with = "optional_error_to_string")]
    pub error: Option<::failure::Error>,
    pub id: u32,
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
            id: build.id,
            stdout: build.stdout,
            stderr: build.stderr,
        }
    }
}
