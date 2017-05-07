use std::fmt::{Debug, Formatter, Result};
use std::cmp;

#[derive(Clone)]
pub struct Channel {
    parts: Vec<String>,
    raw: String,
}

lazy_static! {
    pub static ref REGISTER_LISTENER: Channel = Channel::from_string("client.listener.register");
    pub static ref FORGET_LISTENER: Channel = Channel::from_string("client.listener.forget");
    pub static ref LIST_CLIENTS: Channel = Channel::from_string("client.list");
    pub static ref IDENTIFY: Channel = Channel::from_string("client.identify");
    pub static ref REPLY: Channel = Channel::from_string("reply");
    pub static ref ERROR: Channel = Channel::from_string("error");
}

impl Debug for Channel {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Channel({:?})", self.raw)
    }
}

impl ToString for Channel {
    fn to_string(&self) -> String {
        self.raw.clone()
    }
}

impl PartialEq<String> for Channel {
    fn eq(&self, other: &String) -> bool {
        self.raw.eq(other)
    }
}

impl PartialEq<str> for Channel {
    fn eq(&self, other: &str) -> bool {
        self.raw.eq(other)
    }
}

impl Channel {
    pub fn from_string<T: ToString>(string: T) -> Channel {
        Channel {
            parts: string.to_string()
                .split('.')
                .map(String::from)
                .collect(),
            raw: string.to_string(),
        }
    }

    pub fn is(&self, other: &Option<Channel>) -> bool {
        if let &Some(ref other) = other {
            self.raw.eq(&other.raw)
        } else {
            false
        }
    }

    pub fn matches(&self, other: &Channel) -> bool {
        for i in 0..cmp::max(self.parts.len(), other.parts.len()) {
            if i == self.parts.len() || i == other.parts.len() {
                return false;
            }
            if self.parts[i] == "*" {
                if i == other.parts.len() - 1 || i == self.parts.len() - 1 {
                    return true;
                }
                continue;
            }
            if other.parts[i] == "*" {
                if i == self.parts.len() - 1 {
                    return true;
                }
                continue;
            }
            if self.parts[i] != other.parts[i] {
                return false;
            }
        }
        true
    }
}
