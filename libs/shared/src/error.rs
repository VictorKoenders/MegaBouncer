use std::convert::From;
use std::error::Error as StdError;
use std::fmt::{Formatter, Display, Result as FmtResult};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub description: String,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    ClientNotReadable { address: String },
    NotImplemented { address: String },
    CouldNotGetIndex { index: usize, ident: String },
    InvalidJson { message: String },
    IOError(::std::io::Error),
    AddrParseError(::std::net::AddrParseError),
    JsonError(::serde_json::Error),
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:?}", self)
    }
}

impl Error {
    pub fn new_client_not_readable(address: String) -> Error {
        Error {
            description: format!("Client {:?} is not readable", address),
            kind: ErrorKind::ClientNotReadable { address: address },
        }
    }
    pub fn new_not_implemented(name: &str) -> Error {
        Error {
            description: format!("Method {:?} is not implemented", name),
            kind: ErrorKind::NotImplemented { address: name.to_owned() },
        }
    }
    pub fn new_could_not_get_index(index: usize, ident: &str) -> Error {
        Error {
            description: format!("Could not get index {} of list {:?}", index, ident),
            kind: ErrorKind::CouldNotGetIndex { index: index, ident: ident.to_owned() },
        }
    }

    pub fn new_invalid_json<T: ToString>(message: T) -> Error {
        let message = message.to_string();
        Error {
            description: format!("Invalid json: {}", message.clone()),
            kind: ErrorKind::InvalidJson { message: message }
        }
    }
}

macro_rules! implement_from {
    ($t:ty, $kind:ident, $description:expr) => {
        
        impl From<$t> for Error {
            fn from(error: $t) -> Error {
                Error {
                    description: format!($description, error),
                    kind: ErrorKind::$kind (error)
                }
            }
        }

    }
}

implement_from!(::std::io::Error, IOError, "IO exception {:?}");
implement_from!(::std::net::AddrParseError, AddrParseError, "Could not parse address {:?}");
implement_from!(::serde_json::Error, JsonError, "Could not convert json {:?}");
