extern crate mio_child_process;
extern crate shared;

use mio_child_process::{CommandAsync, Process};
use std::sync::mpsc::TryRecvError;
use std::process::{Command, Stdio};
use shared::mio_poll_wrapper::Handle;
use shared::mio::{Token, Event};
use shared::serde_json::Value;
use std::collections::HashMap;

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

fn token_listener(state: &mut State, _handle: &mut Handle, token: Token, _event: Event) {
    let mut should_remove = false;
    if let Some(process) = state.processes.get_mut(&token) {
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
        state.processes.remove(&token);
    }
}

fn launch_node(state: &mut State, handle: &mut Handle, _: &str, value: &Value) {
    println!("Launching node {:?}", value["node_name"]);
    if let Some(name) = value["node_name"].as_str() {
        let mut dir = ::std::env::current_dir().unwrap();
        dir.push(name);
        let process = Command::new("cargo")
            .arg("run")
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn_async()
            .expect("Could not spawn process");
        let token = handle.register(&process).expect("Could not register process");
        state.processes.insert(token, (name.to_owned(), process));
    }
}
