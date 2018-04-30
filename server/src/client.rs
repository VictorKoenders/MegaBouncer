use shared::mio::net::TcpStream;
use shared::mio::{Poll, Ready, Token};
use shared::semver::Version;
use shared::serde_json::Value;
use shared::Reader;
use std::fmt;
use std::net::SocketAddr;
use uuid::Uuid;

pub struct Client {
    addr: SocketAddr,
    reader: Reader,
    id: Uuid,
    name: Option<String>,
    version: Option<Version>,
}

impl fmt::Debug for Client {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut res = fmt.debug_struct("Client");
        if let Some(ref name) = self.name {
            res.field("name", &name);
        }
        if let Some(ref version) = self.version {
            res.field("version", &version);
        }
        res.field("id", &self.id.to_string())
            .field("addr", &self.addr.to_string())
            .finish()
    }
}

impl Client {
    pub fn new(stream: TcpStream, addr: SocketAddr, poll: &Poll, token: Token) -> Client {
        Client {
            addr,
            reader: Reader::new(stream, poll, token),
            id: Uuid::new_v4(),
            name: None,
            version: None,
        }
    }

    pub fn is(&self, token: Token) -> bool {
        self.reader.is(token)
    }

    pub fn update(&mut self, kind: Ready) -> Result<Vec<Value>, String> {
        let mut result: Vec<Value> = Vec::new();
        if kind.is_readable() {
            result.append(&mut self.reader.read()?);
        }
        Ok(result)
    }
}
