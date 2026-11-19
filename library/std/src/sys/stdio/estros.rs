use crate::io;
use crate::sync::OnceLock;
use crate::sys::cap::Capability;
use crate::sys::cap_table;

#[expect(dead_code)]
#[path = "unsupported.rs"]
mod unsupported_stdio;

pub type Stdin = unsupported_stdio::Stdin;
pub const STDIN_BUF_SIZE: usize = unsupported_stdio::STDIN_BUF_SIZE;

pub struct Stdout;
pub struct Stderr;

impl Stdout {
    pub const fn new() -> Self {
        Self
    }
}

impl Stderr {
    pub const fn new() -> Self {
        Self
    }
}

static STDOUT_CAP: OnceLock<Option<Capability>> = OnceLock::new();
static STDERR_CAP: OnceLock<Option<Capability>> = OnceLock::new();

fn stdout() -> Option<&'static Capability> {
    STDOUT_CAP.get_or_init(|| cap_table::get_cap_table().get("stdout").cloned()).as_ref()
}

fn stderr() -> Option<&'static Capability> {
    STDERR_CAP.get_or_init(|| cap_table::get_cap_table().get("stderr").cloned()).as_ref()
}

fn write(cap: Option<&Capability>, buf: &[u8]) -> io::Result<usize> {
    let Some(cap) = cap else {
        return Err(io::const_error!(io::ErrorKind::BrokenPipe, "no output capability"));
    };
    if cap.send_message(buf, Vec::new()).is_none() {
        return Err(io::const_error!(io::ErrorKind::BrokenPipe, "failed to send output message"));
    }
    Ok(buf.len())
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        write(stdout(), buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        write(stderr(), buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
