pub extern crate mio;
pub extern crate mio_extras;
pub extern crate mio_poll_wrapper;
pub extern crate serde;
pub extern crate serde_json;

pub mod client;
pub mod client_state;
pub mod linereader;
pub mod messages;

use mio::{Event, Token};
use mio_poll_wrapper::Handle;
use serde_json::Value;

/// Checks if the given channel matches any of the channels in the list
/// Channels match if:
/// - They are an exact match, e.g. "test" and "test"
/// - The given channel has more sub-parts, e.g. "test.test" matches "test"
/// - The given channel has a wildcard, e.g. "test.test.abc" matches "test.*.abc"
pub fn listening_to<T: AsRef<str>>(channels: &[T], action: &str) -> bool {
    let mut action_split = action.split('.');
    'outer: for c in channels {
        if c.as_ref() == action {
            return true;
        }
        let mut split = c.as_ref().split('.');
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

pub struct Startup<'a, T: 'a> {
    pub handle: &'a mut Handle,
    pub emit: Vec<Value>,
    pub state: &'a mut T,
}

pub struct ChannelUpdate<'a, T: 'a> {
    pub channel: &'a str,
    pub value: &'a Value,
    pub emit: Vec<Value>,
    pub reply: Vec<Value>,
    pub handle: &'a mut Handle,
    pub state: &'a mut T,
}

pub struct TokenUpdate<'a, T: 'a> {
    pub handle: &'a mut Handle,
    pub state: &'a mut T,
    pub emit: Vec<Value>,
    pub token: Token,
    pub event: Event,
}

#[test]
fn test_client_listener() {
    assert_eq!(false, listening_to::<String>(&[], ""));
    assert_eq!(false, listening_to(&["test".to_string()], "asd"));
    assert_eq!(true, listening_to(&["test".to_string()], "test"));
    assert_eq!(true, listening_to(&["test".to_string()], "test.test"));
    assert_eq!(true, listening_to(&["test".to_string()], "test.asd"));
    assert_eq!(false, listening_to(&["test.*".to_string()], "test"));
    assert_eq!(true, listening_to(&["test.*".to_string()], "test.asd"));
    assert_eq!(true, listening_to(&["test.*".to_string()], "test.asd.asd"));
    assert_eq!(
        true,
        listening_to(&["test.*.asd".to_string()], "test.asd.asd")
    );
    assert_eq!(false, listening_to(&["test.test".to_string()], "test.asd"));
}
