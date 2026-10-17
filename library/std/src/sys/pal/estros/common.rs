use crate::io as std_io;

pub unsafe fn init(_argc: isize, _argv: *const *const u8, _sigpipe: u8) {}

pub unsafe fn cleanup() {}

pub fn unsupported<T>() -> std_io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> std_io::Error {
    std_io::Error::UNSUPPORTED_PLATFORM
}

pub fn abort_internal() -> ! {
    asm!("svc #2")
}
