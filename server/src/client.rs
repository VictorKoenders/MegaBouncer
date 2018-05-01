use serde_json::{to_vec, Value};
use std::io::Write;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_io::io::WriteHalf;
use uuid::Uuid;

#[derive(Debug)]
pub struct Client {
    pub id: Uuid,
    pub name: Option<String>,
    address: SocketAddr,
    writer: WriteHalf<TcpStream>,
    pub listening_to: Vec<String>,
}

impl Client {
    pub fn new(address: SocketAddr, writer: WriteHalf<TcpStream>) -> Client {
        Client {
            id: Uuid::new_v4(),
            name: None,
            address,
            writer,
            listening_to: Vec::new(),
        }
    }

    pub fn send(&mut self, message: &Value) {
        let mut bytes = to_vec(message).unwrap();
        bytes.extend(&[b'\r', b'\n']);
        self.writer.write_all(&bytes).unwrap();
    }
    pub fn is_listening_to(&self, action: &str) -> bool {
        listening_to(&self.listening_to, action)
    }
}

fn listening_to(channels: &[String], action: &str) -> bool {
    let mut action_split = action.split('.');
    'outer: for c in channels {
        if c == action {
            return true;
        }
        let mut split = c.split('.');
        'inner: loop {
            let pair = (split.next(), action_split.next());
            match pair {
                (None, Some(_)) => break 'inner,
                (Some("*"), Some(_)) => continue 'inner,
                (Some(x), Some(y)) if x != y => continue 'outer,
                (Some(x), Some(y)) if x == y => continue 'inner,
                (None, None) => break 'inner,
                _ => continue 'outer,
            }
        }
        return true;
    }
    false
}

#[test]
fn test_client_listener() {
    assert_eq!(false, listening_to(&[], ""));
    assert_eq!(false, listening_to(&["test".to_string()], "asd"));
    assert_eq!(true, listening_to(&["test".to_string()], "test"));
    assert_eq!(true, listening_to(&["test".to_string()], "test.test"));
    assert_eq!(true, listening_to(&["test".to_string()], "test.asd"));
    assert_eq!(true, listening_to(&["test.*".to_string()], "test.asd"));
    assert_eq!(true, listening_to(&["test.*".to_string()], "test.asd.asd"));
    assert_eq!(false, listening_to(&["test.test".to_string()], "test.asd"));
}
