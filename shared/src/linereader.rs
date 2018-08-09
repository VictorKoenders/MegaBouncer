use std::io::{Error, ErrorKind, Read};
use std::{ptr, str};

/// Contains a wrapper over a reader that will split the data in lines
pub struct LineReader<R> {
    /// The reader
    reader: R,
    /// The position of the current buffer
    buffer_position: usize,
    /// The current buffer
    buffer: [u8; 1024],
}

impl<R: Read> LineReader<R> {
    /// Create a new LineReader over a given reader
    pub fn new(reader: R) -> LineReader<R> {
        LineReader {
            reader,
            buffer_position: 0,
            buffer: [0u8; 1024],
        }
    }

    pub fn read_line(&mut self) -> Result<Option<String>, Error> {
        loop {
            if let Some(index) = self.buffer[..self.buffer_position]
                .iter()
                .position(|c| *c == b'\n')
            {
                let str = str::from_utf8(&self.buffer[..index])
                    .unwrap()
                    .trim()
                    .to_string();
                unsafe {
                    ptr::copy(
                        &self.buffer[index + 1],
                        &mut self.buffer[0],
                        self.buffer_position - index - 1,
                    );
                }
                self.buffer_position -= index + 1;
                return Ok(Some(str));
            }
            let n = match self.reader.read(&mut self.buffer[self.buffer_position..]) {
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    return Ok(None);
                }
                Err(e) => return Err(e),
                Ok(0) => return Err(Error::from(ErrorKind::UnexpectedEof)),
                Ok(n) => n,
            };
            self.buffer_position += n;
            if self.buffer_position == self.buffer.len() {
                println!("Buffer overflowed");
                return Err(Error::from(ErrorKind::InvalidData));
            }
        }
    }
}
