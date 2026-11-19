use crate::io;
use crate::sys::cap::Capability;
use crate::sys::syscalls;

/// TODO: doc
pub fn spawn(elf: &[u8], caps: &[Capability]) -> io::Result<u64> {
    let ids: Vec<u64> = caps.iter().map(Capability::id).collect();
    let pid = unsafe {
        syscalls::spawn_process(elf.as_ptr(), elf.len(), ids.as_ptr() as *const u8, ids.len())
    };
    if (pid as i64) < 0 {
        return Err(io::const_error!(io::ErrorKind::Other, "failed to spawn process"));
    }
    Ok(pid)
}
