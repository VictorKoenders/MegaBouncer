use std::rc::Rc;
use std::process::Child;

pub struct Project {
    pub directory: String,
    pub builds: Vec<Rc<Build>>,
    pub build_process: Option<Child>,
    pub running_process: Option<Child>,
}

pub struct Build {
    pub name: String,
    pub directory: String,
    pub pattern: String,
    pub build: BuildType,
    pub after_success: Vec<PostBuildEvent>,
}

pub enum BuildType {
    Cargo,
    TypescriptReactWebpack,
    Custom {
        command: String,
        args: Vec<String>,
    }
}

pub enum PostBuildEvent {
    TriggerBuild {
        name: String,
    },
    Run(RunType),
}

pub enum RunType {
    Cargo,
    TypescriptReactWebpack,
    Custom {
        command: String,
        args: Vec<String>,
    }
}