use mio::*;
use serde_json::{self, Value};
use std::error::Error;
use std::io::Read;
use std::collections::vec_deque::VecDeque;
use std::fmt;

pub struct Reader {
    stream: net::TcpStream,
    token: Token,
    buffer: VecDeque<u8>,
}

impl fmt::Debug for Reader {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Client {:?}", self.token)
    }
}

const BUFFER_LENGTH: usize = 1024;

impl Reader {
    pub fn new(stream: net::TcpStream, poll: &Poll, token: Token) -> Reader {
        let reader = Reader {
            stream,
            token,
            buffer: VecDeque::with_capacity(BUFFER_LENGTH),
        };
        poll.register(
            &reader.stream,
            reader.token,
            Ready::readable() | Ready::writable(),
            PollOpt::edge(),
        ).unwrap();
        reader
    }

    pub fn is(&self, token: Token) -> bool {
        self.token == token
    }

    pub fn read(&mut self) -> Result<Vec<Value>, String> {
        let mut result = Vec::new();
        loop {
            let mut bytes = [0; BUFFER_LENGTH];
            let length = match self.stream.read(&mut bytes) {
                Ok(l) => l,
                Err(e) => {
                    if e.kind() == ::std::io::ErrorKind::WouldBlock {
                        break;
                    } else {
                        return Err(e.description().to_owned());
                    }
                }
            };
            self.buffer.extend(&bytes[..length]);
            while let Some(index) = self.buffer.iter().position(|c| *c == b'\n') {
                let drained = self.buffer.drain(..index + 1).take(index).collect::<Vec<_>>();
                let value: Value = match serde_json::from_slice(&drained) {
                    Ok(v) => v,
                    Err(e) => return Err(format!("Could not parse JSON: {:?}", e)),
                };

                result.push(value);
            }
        }
        if cfg!(debug) {
            if self.buffer.capacity() > BUFFER_LENGTH {
                println!("Buffer is larger than expectd! {} > {}", self.buffer.capacity(), BUFFER_LENGTH);
            }
            if self.buffer.len() > 0 {
                println!("Buffer is not empty after finishing Reader::read");
            }
        }
        Ok(result)
    }
}
