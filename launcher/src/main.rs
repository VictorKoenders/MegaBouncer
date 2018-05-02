extern crate shared;

use shared::serde_json::Value;

fn main() {
    let mut client = shared::client::Client::new("Launcher", ());
    client.register_listener("node.launch", launch_node);
    client.launch();
}

fn launch_node(_: &mut (), _: &str, value: &Value) {
    println!("Launching node {:?}", value["node_name"]);
    if let Some(name) = value["node_name"].as_str() {
        let mut dir = ::std::env::current_dir().unwrap();
        dir.push(name);
        println!("{:?}", dir);
        let command = std::process::Command::new("cargo")
                        .arg("run")
                        .current_dir(dir)
                        .output()
                        .expect("Could not run process");
    }
}
