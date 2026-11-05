use crate::io as std_io;
use core::arch::asm;

pub unsafe fn init(_argc: isize, _argv: *const *const u8, _sigpipe: u8) {}

pub unsafe fn cleanup() {}

pub fn unsupported<T>() -> std_io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> std_io::Error {
    std_io::Error::UNSUPPORTED_PLATFORM
}

pub fn abort_internal() -> ! {
    unsafe { asm!("svc #2", options(noreturn)) }
}
