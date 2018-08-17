extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate serde_json;
extern crate shared;

mod web_server;

#[cfg(target_os = "linux")]
const OPEN_COMMAND: &str = "xdg-open";
#[cfg(target_os = "windows")]
const OPEN_COMMAND: &str = "start";

fn main() {
    let url = web_server::serve();
    std::process::Command::new(OPEN_COMMAND)
        .arg(&url)
        .spawn()
        .unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
