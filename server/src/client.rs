use serde_json::{to_vec, Value};
use std::io::Write;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_io::io::WriteHalf;
use uuid::Uuid;

/// Holds a reference to a single connected TCP client
#[derive(Debug)]
pub struct Client {
    /// Random ID of the client
    pub id: Uuid,
    /// The name of a client, if any
    pub name: Option<String>,
    /// The remote address of the client
    address: SocketAddr,
    /// The writer that is associated with the TcpStream
    writer: WriteHalf<TcpStream>,
    /// A list of channels that this client is listening to
    pub listening_to: Vec<String>,
}

impl Client {
    /// Create a new client with the given address and writer
    /// an ID will be automatically generated, and the client will not be listening to anything
    pub fn new(address: SocketAddr, writer: WriteHalf<TcpStream>) -> Client {
        Client {
            id: Uuid::new_v4(),
            name: None,
            address,
            writer,
            listening_to: Vec::new(),
        }
    }

    /// Send a given JSON message to a client
    /// This function is blocking
    pub fn send(&mut self, message: &Value) {
        // TODO: Figure out how to make this function non-blocking
        let mut bytes = to_vec(message).unwrap();
        bytes.extend(&[b'\r', b'\n']);
        self.writer.write_all(&bytes).unwrap();
    }

    /// Checks if the client is listening to the given channel
    pub fn is_listening_to(&self, action: &str) -> bool {
        listening_to(&self.listening_to, action)
    }
}

/// Checks if the given channel matches any of the channels in the list
/// Channels match if:
/// - They are an exact match, e.g. "test" and "test"
/// - The given channel has more sub-parts, e.g. "test.test" matches "test"
/// - The given channel has a wildcard, e.g. "test.test.abc" matches "test.*.abc"
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
    assert_eq!(false, listening_to(&["test.*".to_string()], "test"));
    assert_eq!(true, listening_to(&["test.*".to_string()], "test.asd"));
    assert_eq!(true, listening_to(&["test.*".to_string()], "test.asd.asd"));
    assert_eq!(true, listening_to(&["test.*.asd".to_string()], "test.asd.asd"));
    assert_eq!(false, listening_to(&["test.test".to_string()], "test.asd"));
}
