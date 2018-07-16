use serde_json;
use std::fs::{read_dir, File};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct Module {
    pub name: String,
    pub path: String,
    pub commands: Vec<Command>,
    pub dependant_upon: Vec<String>,
}

impl Module {
    pub fn from_file(file: &PathBuf) -> Module {
        let reader = File::open(&file).unwrap();
        let mut module: Module = serde_json::from_reader(reader).unwrap();
        let file = file.to_str().unwrap().to_string();
        module.name = file.clone();
        module.path = file;
        module
    }

    pub fn get_modules() -> Vec<Module> {
        let dirs = read_dir(".").unwrap();
        let mut result = Vec::new();
        for dir in dirs {
            let dir = dir.unwrap();
            if !dir.file_type().unwrap().is_dir() {
                continue;
            }
            let file_name = dir.file_name();
            let file_name = file_name.to_str().unwrap();
            if file_name.starts_with('.') {
                continue;
            }
            if BLACKLISTED_DIRS.iter().any(|d| d == &file_name) {
                continue;
            }

            let module_file = Path::new(file_name).join("module_info.json");
            let module = if module_file.exists() {
                Module::from_file(&module_file)
            } else {
                let mut dependant_upon = vec![];
                if file_name != "shared" {
                    dependant_upon.push("shared".to_string());
                }
                if file_name != "server" && file_name != "shared" {
                    dependant_upon.push("server".to_string());
                }
                Module {
                    name: file_name.to_string(),
                    path: file_name.to_string(),
                    commands: Command::get_suggested_commands(file_name),
                    dependant_upon,
                }
            };
            result.push(module);
        }
        result
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct Command {
    pub name: String,
    pub filter: String,
    pub command: String,
    pub directory: String,
    pub trigger_command_after_done: Option<String>,
}

impl Command {
    pub fn get_suggested_commands(path: &str) -> Vec<Command> {
        let path = Path::new(path);
        let mut result = Vec::new();
        if path.join("Cargo.toml").exists() {
            result.push(Command {
                name: String::from("rust"),
                filter: String::from("*.rs,*.toml"),
                command: if path.to_str() != Some("shared") {
                    String::from("cargo run")
                } else {
                    String::from("cargo build")
                },
                directory: String::from(""),
                trigger_command_after_done: None,
            });
        }
        if path.join("ui").join("package.json").exists() {
            let has_rust = result.iter().any(|c| c.name == "rust");
            result.push(Command {
                name: String::from("webpack"),
                filter: String::from("*.js,*.tsx,*.ts"),
                command: String::from("webpack.cmd"),
                directory: String::from("ui"),
                trigger_command_after_done: if has_rust {
                    Some(String::from("rust"))
                } else {
                    None
                },
            });
        }
        result
    }
}

impl Module {}

const BLACKLISTED_DIRS: [&'static str; 4] = ["libs", "target", "test_tool", "launcher"];
