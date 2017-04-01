use std::collections::VecDeque;
use std::convert::TryInto;
use mio::tcp::TcpStream;
use std::io::Write;

pub struct WriteQueue<'a, T: TryInto<Vec<u8>> + Sized + 'static> {
    pub byte_queue: &'a mut Vec<u8>,
    pub message_queue: &'a mut VecDeque<T>,
    pub stream: &'a mut TcpStream,
    pub writeable: &'a mut bool,
}

pub trait Writeable<T: TryInto<Vec<u8>> + Sized + 'static> 
    where <T as TryInto<Vec<u8>>>::Error: ::std::fmt::Debug {
    fn get_write_queue(&mut self) -> WriteQueue<T>;
    fn try_write(&mut self) {
        let write_queue = self.get_write_queue();
        if !*write_queue.writeable { return; }
        loop {
            if !write_queue.byte_queue.is_empty() {
                match write_queue.stream.write(write_queue.byte_queue) {
                    Ok(length) => {
                        write_queue.byte_queue.drain(..length);
                        if !write_queue.byte_queue.is_empty() {
                            println!("Could not write full queue, retrying again next time");
                            *write_queue.writeable = false;
                            return;
                        }
                    },
                    Err(e) => {
                        println!("Could not write to stream: {:?}", e);
                        *write_queue.writeable = false;
                        return;
                    }
                }
            } else if let Some(message) = write_queue.message_queue.pop_front() {
                match message.try_into() {
                    Ok(bytes) => {
                        write_queue.byte_queue.extend(bytes.into_iter());
                        write_queue.byte_queue.extend(b"\n");
                    },
                    Err(e) => {
                        println!("Could not convert message into bytes: {:?}", e);
                    }
                };
            } else {
                break;
            }
        }
    }
}

