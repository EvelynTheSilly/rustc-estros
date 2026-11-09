use crate::io;
use crate::io::BufRead;
use crate::io::Read;
use crate::sys::cap::Message;

#[stable(feature = "estros", since = "1.0.0")]
pub struct MessageReader {
    message: Message,
    buffer: Vec<u8>,
    pos: usize,
    len: usize,
}

impl MessageReader {
    /// TODO: DOC
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn new(message: Message) -> Self {
        Self { message, buffer: Vec::new(), pos: 0, len: 0 }
    }

    /// TODO: DOC
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn into_inner(self) -> Message {
        self.message
    }
}

#[stable(feature = "estros", since = "1.0.0")]
impl BufRead for MessageReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.pos >= self.len {
            self.pos = 0;

            // Allocate only when we actually need to read.
            if self.buffer.is_empty() {
                self.buffer.resize(4096, 0);
            }

            self.len = self
                .message
                .read_bytes(&mut self.buffer.as_mut_slice())
                .ok_or_else(|| io::Error::other("message read failed"))?;
        }

        Ok(&self.buffer.as_mut_slice()[self.pos..self.len])
    }

    fn consume(&mut self, amount: usize) {
        self.pos = self.pos.saturating_add(amount).min(self.len);
    }
}

#[stable(feature = "estros", since = "1.0.0")]
impl Read for MessageReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let available = self.fill_buf()?;

        let amount = available.len().min(buf.len());

        buf[..amount].copy_from_slice(&available[..amount]);

        self.consume(amount);

        Ok(amount)
    }
}
