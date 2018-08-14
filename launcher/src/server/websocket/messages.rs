use actix::Addr;
use super::WebsocketClient;

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub client_addr: Addr<WebsocketClient>,
}

#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
pub struct BroadcastStateChange(pub String);

/*
#[derive(Debug, Message)]
pub enum ServerToClient {
    StateChange(StateChange),
    State(Value),
}

#[derive(Debug, Message)]
pub enum ClientToServer {
    Connect,
    TriggerBuild {
        project_name: String,
        build_name: String,
    },
    KillProcess {
        pid: u32,
    },
    Disconnect,
}
*/