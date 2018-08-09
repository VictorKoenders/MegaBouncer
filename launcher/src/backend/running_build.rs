use super::{Build, RunType};
use chrono::{DateTime, Utc};
use mio::Token;
use mio_child_process::{CommandAsync, Process};
use std::process::Stdio;
use Result;

pub struct RunningBuild {
    pub project_name: String,
    pub build: Build,
    pub started_on: DateTime<Utc>,
    pub token: Token,
    pub process: Process,
}

impl RunningBuild {
    pub fn new(project_name: String, build: Build, token: Token) -> Result<RunningBuild> {
        let mut command = build.build.create_command();
        command.current_dir(&build.directory);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let process = command.spawn_async()?;

        Ok(RunningBuild {
            project_name,
            build,
            started_on: Utc::now(),
            token,
            process,
        })
    }
}

pub struct RunningProcess {
    pub project_name: String,
    pub build: Build,
    pub run_type: RunType,
    pub token: Token,
    pub process: Process,
}

impl RunningProcess {
    pub fn new(
        project_name: String,
        build: Build,
        run_type: RunType,
        token: Token,
    ) -> Result<RunningProcess> {
        let mut command = run_type.create_command();
        command.current_dir(&build.directory);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let process = command.spawn_async()?;

        Ok(RunningProcess {
            project_name,
            build,
            run_type,
            token,
            process,
        })
    }
}
