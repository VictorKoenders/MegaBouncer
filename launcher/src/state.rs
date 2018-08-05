use std::sync::Mutex;
use mio_extras::channel::{Sender, channel};
use backend::BackendRequest;

#[derive(Clone)]
pub struct State {
    pub modules: Vec<StateModule>,
    pub processes: Vec<StateProcess>,
    pub sender: Sender<BackendRequest>,
}

lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(State {
        modules: Vec::new(),
        processes: Vec::new(),
        sender: channel().0,
    });
}

impl State {
    pub fn get<F>(cb: F)
        where F : FnOnce(&State) {
        let state = STATE.lock().unwrap();
        cb(&state);
    }

    pub fn modify<F>(cb: F) 
        where F : FnOnce(&mut State) {
        let mut state = STATE.lock().unwrap();
        cb(&mut state);
    }
}

#[derive(Debug, Clone)]
pub struct StateModule {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct StateProcess {
    pub module: StateModule,
    pub status: String,
}
