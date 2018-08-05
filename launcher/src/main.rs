extern crate actix;
extern crate actix_web;
extern crate mio;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sysinfo;
#[macro_use]
extern crate lazy_static;
extern crate mio_child_process;
extern crate mio_extras;
extern crate shared;

mod state;
mod server;
mod backend;

use std::thread::spawn;

fn main() {
    spawn(server::run);
    backend::run();
}
