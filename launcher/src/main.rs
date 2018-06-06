extern crate mio_child_process;
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
