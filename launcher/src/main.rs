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
extern crate chrono;
extern crate clap;
extern crate mio_child_process;
extern crate mio_extras;
extern crate shared;
#[macro_use]
extern crate failure;
extern crate uuid;

mod backend;
mod server;
mod state;

use clap::{App, Arg};
use std::thread::spawn;

pub type Result<T> = std::result::Result<T, failure::Error>;

fn main() {
    let matches = App::new("Launcher")
        .arg(
            Arg::with_name("base_dir")
                .short("d")
                .long("base_dir")
                .help("Set the base directory of the projects")
                .required(true)
                .takes_value(true),
        ).get_matches();

    spawn(server::run);
    if let Err(e) = backend::run(matches.value_of("base_dir").unwrap()) {
        println!("Backend failed: {:?}", e);
        std::process::exit(-1);
    }
}
