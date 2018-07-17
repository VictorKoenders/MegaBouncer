#![allow(warnings)]

#[macro_use]
extern crate serde_derive;
extern crate mio;
extern crate mio_child_process;
extern crate serde;
extern crate serde_json;

mod module;

use mio::{Events, Poll, PollOpt, Ready, Token};
use mio_child_process::{CommandAsync, Process as ChildProcess, ProcessEvent, StdioChannel};
use module::{Command, Module};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::{Command as Process, Stdio};
use std::rc::Rc;
use std::sync::mpsc::TryRecvError;

fn main() {
    let mut arg = std::env::args();
    arg.next();
    let mut programs = arg.collect::<Vec<_>>();
    let all_modules = Module::get_modules();
    if programs.is_empty() || programs.iter().any(|p| p == "--help" || p == "-h") {
        println!("Usage: ");
        println!("  -h/--help   Print this help");
        println!("  all         Run all modules");
        println!("  <modules>   Run specified modules (see below)");
        println!();
        println!("Available modules:");
        for module in all_modules {
            println!("  {}", module.name);
        }
        return;
    }
    let run_all = if let Some(p) = programs.iter().position(|p| p == "all") {
        programs.remove(p);
        true
    } else {
        false
    };
    let modules_to_run = all_modules
        .into_iter()
        .filter(|m| {
            if run_all {
                true
            } else {
                let position = programs.iter().position(|p| p == &m.name);
                if let Some(p) = position {
                    programs.remove(p);
                    true
                } else {
                    false
                }
            }
        })
        .collect::<Vec<_>>();
    if !programs.is_empty() {
        println!("Could not find programs: {:?}", programs);
        return;
    }

    let mut token_index = 1;
    let mut modules = Vec::new();
    for module in &modules_to_run {
        for command in &module.commands {
            modules.push(Rc::new(RefCell::new(RunningModule::new(
                Token(token_index),
                module,
                command,
            ))));
            token_index += 1;
        }
    }

    for (index, module) in modules.iter().enumerate() {
        let mut module_ref = module.borrow_mut();
        for dependant in &module_ref.module.dependant_upon {
            let other = {
                let mut range = modules[..index].iter().chain(modules[index + 1..].iter());
                range
                    .find(|m| &m.borrow().module.name == dependant)
                    .unwrap()
            };
            let mut other_ref = other.borrow_mut();
            other_ref.reverse_dependant_upon.push(module.clone());
            module_ref.dependant_upon.push(other.clone());
        }
    }

    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(10);
    start_task(&mut modules, &poll);
    while poll.poll(&mut events, None).is_ok() {
        for event in &events {
            let mut should_start_task = false;
            for module in &modules {
                let mut module_ref = module.borrow_mut();
                if module_ref.token != event.token() {
                    continue;
                }
                let result = handle_child_update(&mut module_ref, &poll);

                if result.did_finish {
                    module_ref.should_build = false;
                    should_start_task = true;
                }
                if result.should_remove {
                    module_ref.process = None;
                }
            }
            if should_start_task {
                start_task(&mut modules, &poll);
            }
        }
    }
}

#[derive(Default)]
struct ChildProcessUpdate {
    did_finish: bool,
    should_remove: bool,
}

fn handle_child_update(module: &mut RunningModule, poll: &Poll) -> ChildProcessUpdate {
    let mut update = ChildProcessUpdate::default();

    let command = &module.command.command;
    let module_name = &module.module.name;
    let mut stdout = &mut module.stdout;
    let mut stderr = &mut module.stderr;
    let mut process = module.process.as_mut().unwrap();
    loop {
        let result = match process.try_recv() {
            Ok(r) => r,
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => panic!("Could not receive from process"),
        };
        match result {
            ProcessEvent::Data(StdioChannel::Stdout, msg) => {
                *stdout += &msg;
                while let Some(i) = stdout.find('\n') {
                    println!("[{} stdout] {}", module_name, &stdout[..i]);
                    stdout.drain(..i+1);
                }
            }
            ProcessEvent::Data(StdioChannel::Stderr, msg) => {
                *stderr += &msg;
                while let Some(i) = stderr.find('\n') {
                    {
                        let line = &stderr[..i];
                        if command == "cargo run" && line.starts_with("    Finished dev [unoptimized + debuginfo] target(s) in ")
                        {
                            update.did_finish = true;
                        }
                        println!("[{} stderr] {}", module_name, &stderr[..i]);
                    }
                    stderr.drain(..i+1);
                }
            }
            ProcessEvent::Exit(exit_status) => {
                println!("{} finished ({:?})", module_name, exit_status);
                poll.deregister(process);
                update.did_finish = true;
                update.should_remove = true;
                break;
            }
            ProcessEvent::CommandError(e) => {
                println!("Command error in {}", module_name);
                println!("{}", e);
            }
            ProcessEvent::IoError(channel, e) => {
                println!(
                    "IO error in {} of {}",
                    if let StdioChannel::Stderr = channel {
                        "stderr"
                    } else {
                        "stdout"
                    },
                    module_name
                );
                println!("{}", e);
            }
            ProcessEvent::Utf8Error(channel, e) => {
                println!(
                    "UTF8 error in {} of {}",
                    if let StdioChannel::Stderr = channel {
                        "stderr"
                    } else {
                        "stdout"
                    },
                    module_name
                );
                println!("{}", e);
            }
        }
    }
    update
}

fn start_task(modules: &mut [Rc<RefCell<RunningModule>>], poll: &Poll) {
    for module in modules {
        let mut module_ref = module.borrow_mut();
        if module_ref.should_build && module_ref.process.is_none() {
            if module_ref
                .dependant_upon
                .iter()
                .map(|r| r.borrow())
                .any(|d| d.should_build)
            {
                continue;
            }

            if module_ref
                .dependant_upon
                .iter()
                .map(|r| r.borrow())
                .any(|d| d.should_build)
            {
                // Dependant modules are not build
                continue;
            }
            let mut dir = PathBuf::from(&module_ref.module.path);
            if !module_ref.command.directory.is_empty() {
                dir.push(&module_ref.command.directory);
            }
            print!("Building {}", module_ref.module.name);
            println!(
                ", running {:?} in ./{}/",
                module_ref.command.command,
                dir.to_str().unwrap(),
            );
            let mut command = module_ref.command.command.split(' ');
            let mut process = Process::new(command.next().unwrap());
            process.current_dir(dir);
            process.stdout(Stdio::piped());
            process.stderr(Stdio::piped());
            process.args(command.collect::<Vec<_>>());
            match process.spawn_async() {
                Ok(child) => {
                    poll.register(&child, module_ref.token, Ready::all(), PollOpt::edge())
                        .unwrap();
                    module_ref.process = Some(child);
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }
    }
}

pub struct RunningModule<'a> {
    token: Token,
    module: &'a Module,
    command: &'a Command,
    dependant_upon: Vec<Rc<RefCell<RunningModule<'a>>>>,
    reverse_dependant_upon: Vec<Rc<RefCell<RunningModule<'a>>>>,
    should_build: bool,
    process: Option<ChildProcess>,
    stdout: String,
    stderr: String,
}

impl<'a> RunningModule<'a> {
    pub fn new(token: Token, module: &'a Module, command: &'a Command) -> RunningModule<'a> {
        RunningModule {
            token,
            module,
            command,
            dependant_upon: Vec::new(),
            reverse_dependant_upon: Vec::new(),
            should_build: true,
            process: None,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

/*extern crate mio_child_process;
extern crate shared;

use mio_child_process::{CommandAsync, Process};
use shared::mio::Token;
use shared::{ChannelUpdate, TokenUpdate};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::mpsc::TryRecvError;

fn main() {
    let mut client = shared::client::Client::new("Launcher", State::default());
    client.register_listener("node.launch", launch_node);
    client.set_token_listener(token_listener);
    client.launch();
}

#[derive(Default)]
struct State {
    processes: HashMap<Token, (String, Process)>,
}

fn token_listener(update: &mut TokenUpdate<State>) {
    let mut should_remove = false;
    if let Some(process) = update.state.processes.get_mut(&update.token) {
        println!("Processes updated: {:?}", process.0);
        loop {
            let result = match process.1.try_recv() {
                Ok(r) => r,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    should_remove = true;
                    break;
                }
            };
            println!("{:?}", result);
        }
    }
    if should_remove {
        println!("Process has ended!");
        update.state.processes.remove(&update.token);
    }
}

fn launch_node(update: &mut ChannelUpdate<State>) {
    println!("Launching node {:?}", update.value["node_name"]);
    if let Some(name) = update.value["node_name"].as_str() {
        let mut dir = ::std::env::current_dir().unwrap();
        dir.push(name);
        let process = Command::new("cargo")
            .arg("run")
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn_async()
            .expect("Could not spawn process");
        let token = update
            .handle
            .register(&process)
            .expect("Could not register process");
        update
            .state
            .processes
            .insert(token, (name.to_owned(), process));
    }
}
*/
