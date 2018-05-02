use futures::{Poll, Stream};
use std::{io, ptr, str};
use tokio_io::AsyncRead;

/// Contains a wrapper over a reader that will split the data in lines
pub struct LineReader<R> {
    /// The reader
    reader: R,
    /// The position of the current buffer
    buffer_position: usize,
    /// The current buffer
    buffer: [u8; 1024],
}

impl<R> LineReader<R> {
    /// Create a new LineReader over a given reader
    pub fn new(reader: R) -> LineReader<R> {
        LineReader {
            reader,
            buffer_position: 0,
            buffer: [0u8; 1024],
        }
    }
}

impl<R: AsyncRead> Stream for LineReader<R> {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
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
                return Ok(Some(str).into());
            }
            let n = try_ready!(
                self.reader
                    .poll_read(&mut self.buffer[self.buffer_position..])
            );
            if n == 0 {
                return Ok(None.into());
            }
            self.buffer_position += n;
            if self.buffer_position == self.buffer.len() {
                println!("Buffer overflowed");
                return Ok(None.into());
            }
        }
    }
}
