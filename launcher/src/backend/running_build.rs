use super::{Build, RunType};
use mio_child_process::{Process, CommandAsync};
use mio::Token;
use std::process::Stdio;
use Result;
use std::path::PathBuf;

pub struct RunningBuild {
    pub directory: String,
    pub build: Build,
    pub token: Token,
    pub process: Process,
}

impl RunningBuild {
    pub fn new(base_dir: &str, directory: String, build: Build, token: Token) -> Result<RunningBuild> {
        let mut command = build.build.create_command();
        let mut path = PathBuf::from(base_dir);
        path.push(&directory);
        println!("Running process in {:?}", path);
        command.current_dir(&path);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let process = command.spawn_async()?;

        Ok(RunningBuild {
            directory,
            build,
            token,
            process,
        })
    }
}

pub struct RunningProcess {
    pub directory: String,
    pub _type: RunType,
    pub token: Token,
    pub process: Process,
}

impl RunningProcess {
    pub fn new(base_dir: &str, directory: String, _type: RunType, token: Token) -> Result<RunningProcess> {
        let mut command = _type.create_command();
        let mut path = PathBuf::from(base_dir);
        path.push(&directory);
        println!("Running process in {:?}", path);
        command.current_dir(&path);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let process = command.spawn_async()?;

        Ok(RunningProcess {
            directory,
            _type,
            token,
            process,
        })
    }
}