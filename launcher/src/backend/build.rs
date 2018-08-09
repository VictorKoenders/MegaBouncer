use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub name: String,
    pub builds: Vec<Build>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Build {
    pub name: String,
    pub directory: String,
    pub pattern: String,
    pub build: BuildType,
    pub after_success: Option<PostBuildEvent>,
}

#[derive(Debug, Clone, Serialize)]
pub enum BuildType {
    Cargo,
    TypescriptReactWebpack,
    #[allow(dead_code)]
    Custom { command: String, args: Vec<String> },
}

impl BuildType {
    pub fn create_command(&self) -> Command {
        match self {
            BuildType::Cargo => {
                let mut c = Command::new("cargo");
                c.arg("build");
                c
            }
            BuildType::TypescriptReactWebpack => {
                let mut c = Command::new("node");
                c.arg("../../node_modules/webpack_cli/bin/cli.js");
                c
            }
            BuildType::Custom { command, args } => {
                let mut c = Command::new(command);
                c.args(args);
                c
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum PostBuildEvent {
    TriggerBuild { name: String },
    Run(RunType),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum RunType {
    Cargo,
    #[allow(dead_code)]
    Custom { command: String, args: Vec<String> },
}

impl RunType {
    pub fn create_command(&self) -> Command {
        match self {
            RunType::Cargo => {
                let mut c = Command::new("cargo");
                c.arg("run");
                c
            }
            RunType::Custom { command, args } => {
                let mut c = Command::new(command);
                c.args(args);
                c
            }
        }
    }
}
