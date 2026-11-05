const SYS_NOOP: usize = 0;
const SYS_WRITE: usize = 1;
const SYS_EXIT: usize = 2;
const SYS_READ_MESSAGE: usize = 3;
const SYS_GET_PID: usize = 4;
const SYS_GET_TID: usize = 5;
const SYS_SPAWN_THREAD: usize = 6;
const SYS_KILL_THREAD: usize = 7;
const SYS_WAIT_ON_THREAD: usize = 8;
const SYS_SPAWN_CAPABILITY: usize = 9;
const SYS_WAIT_ON_MESSAGE_FROM_RECEIVER: usize = 10;
const SYS_SEND_MESSAGE_TO_RECEIVER: usize = 11;
const SYS_MAP_DEVICE_MEMORY: usize = 12;
const SYS_SPAWN_PROCESS: usize = 13;
const SYS_CAP_DUPLICATE: usize = 14;
const SYS_LOOKUP_MEM_CAP: usize = 15;
const SYS_READ_MESSAGE_CAPS: usize = 16;

/// runs a syscall
/// example:
/// syscall!(NUMBER, arg1, arg2, arg3, ...);
/// TODO: better docs
macro_rules! syscall {
    ($nr:expr $(, $arg:expr )* $(,)?) => {{
        let ret: usize;
        syscall!(@build $nr, (lateout("x0") ret, options(nostack)) 0 $(, $arg )*);
        ret
    }};

    (@build $nr:expr, ($($tail:tt)*) $idx:tt) => {
        ::core::arch::asm!(
            "svc {nr}",
            nr = const $nr,
            $($tail)*
        )
    };
    (@build $nr:expr, ($($tail:tt)*) $idx:tt,) => {
        ::core::arch::asm!(
            "svc {nr}",
            nr = const $nr,
            $($tail)*
        )
    };
    (@build $nr:expr, ($($tail:tt)*) 0, $a:expr $(, $rest:expr)*) => {
        syscall!(@build $nr, (in("x0") $a, $($tail)*) 1 $(, $rest)*)
    };
    (@build $nr:expr, ($($tail:tt)*) 1, $a:expr $(, $rest:expr)*) => {
        syscall!(@build $nr, (in("x1") $a, $($tail)*) 2 $(, $rest)*)
    };
    (@build $nr:expr, ($($tail:tt)*) 2, $a:expr $(, $rest:expr)*) => {
        syscall!(@build $nr, (in("x2") $a, $($tail)*) 3 $(, $rest)*)
    };
    (@build $nr:expr, ($($tail:tt)*) 3, $a:expr $(, $rest:expr)*) => {
        syscall!(@build $nr, (in("x3") $a, $($tail)*) 4 $(, $rest)*)
    };
    (@build $nr:expr, ($($tail:tt)*) 4, $a:expr $(, $rest:expr)*) => {
        syscall!(@build $nr, (in("x4") $a, $($tail)*) 5 $(, $rest)*)
    };
    (@build $nr:expr, ($($tail:tt)*) 5, $a:expr $(, $rest:expr)*) => {
        syscall!(@build $nr, (in("x5") $a, $($tail)*) 6 $(, $rest)*)
    };
    (@build $nr:expr, ($($tail:tt)*) 6, $a:expr $(, $rest:expr)*) => {
        syscall!(@build $nr, (in("x6") $a, $($tail)*) 7 $(, $rest)*)
    };
}

macro_rules! syscall_fn {
    ($name:ident, $nr:expr $(, $arg:ident : $ty:ty)*) => {
        #[inline]
        #[allow(dead_code, missing_docs)]
        #[stable(feature = "estros", since = "1.0.0")]
        pub unsafe fn $name($($arg: $ty),*) -> usize {
            unsafe { syscall!($nr $(, $arg)*) }
        }
    };
}

syscall_fn!(noop, SYS_NOOP);
syscall_fn!(write_to_uart, SYS_WRITE, ptr: *const u8, len: usize);
syscall_fn!(exit, SYS_EXIT);
syscall_fn!(
    read_message,
    SYS_READ_MESSAGE,
    cap_id: u64,
    mid: u64,
    ptr: *mut u8,
    len: usize
);
syscall_fn!(get_pid, SYS_GET_PID);
syscall_fn!(get_tid, SYS_GET_TID);
syscall_fn!(spawn_thread, SYS_SPAWN_THREAD, location: usize, arg: usize);
syscall_fn!(kill_thread, SYS_KILL_THREAD, tid: u64);
syscall_fn!(wait_on_thread, SYS_WAIT_ON_THREAD, tid: u64);
syscall_fn!(spawn_capability, SYS_SPAWN_CAPABILITY, kind: u64);
syscall_fn!(
    wait_on_message_from_receiver,
    SYS_WAIT_ON_MESSAGE_FROM_RECEIVER,
    cap_id: u64
);
syscall_fn!(
    send_message_to_receiver,
    SYS_SEND_MESSAGE_TO_RECEIVER,
    cap_id: u64,
    ptr: *const u8,
    len: usize,
    cap_ptr: *const u8,
    cap_len: usize
);
syscall_fn!(
    map_device_memory,
    SYS_MAP_DEVICE_MEMORY,
    vaddr: usize,
    cap_id: u64
);
syscall_fn!(
    spawn_process,
    SYS_SPAWN_PROCESS,
    elf_ptr: *const u8,
    elf_len: usize,
    cap_ptr: *const u8,
    cap_len: usize
);
syscall_fn!(cap_duplicate, SYS_CAP_DUPLICATE, cap_id: u64);
syscall_fn!(lookup_mem_cap, SYS_LOOKUP_MEM_CAP, addr: usize, size: usize);
syscall_fn!(
    read_message_caps,
    SYS_READ_MESSAGE_CAPS,
    cap_id: u64,
    mid: u64,
    ptr: *mut u8,
    len: usize
);
