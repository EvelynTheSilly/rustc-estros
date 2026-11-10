use crate::{io, sys::syscalls};

#[expect(dead_code)]
#[path = "unsupported.rs"]
mod unsupported_stdio;

pub type Stdin = unsupported_stdio::Stdin;

pub struct Stdout;
pub type Stderr = Stdout;

impl Stdout {
    pub const fn new() -> Self {
        Self
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
        unsafe {
            syscalls::write_to_uart(buf.as_ptr(), buf.len());
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> crate::io::Result<()> {
        Ok(())
    }
}
pub const STDIN_BUF_SIZE: usize = unsupported_stdio::STDIN_BUF_SIZE;

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
