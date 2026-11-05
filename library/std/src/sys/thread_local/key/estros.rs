use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use core::sync::atomic::{AtomicU32, AtomicUsize};

use crate::sys::syscalls::get_tid;

const MAX_KEYS: usize = 64;
const TID_LIMIT: usize = 2048;
const ROW_SLOTS: usize = 1024;
const MAX_ROWS: usize = ROW_SLOTS / MAX_KEYS;

static NEXT_KEY: AtomicUsize = AtomicUsize::new(0);
static KEY_DTORS: [AtomicUsize; MAX_KEYS] = [const { AtomicUsize::new(0) }; MAX_KEYS];

static NEXT_ROW: AtomicUsize = AtomicUsize::new(0);
static ROW_INDEX: [AtomicU32; TID_LIMIT] = [const { AtomicU32::new(0) }; TID_LIMIT];
static ROWS: [AtomicUsize; ROW_SLOTS] = [const { AtomicUsize::new(0) }; ROW_SLOTS];

pub type Key = usize;

#[inline]
pub fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    let key = NEXT_KEY.fetch_add(1, Relaxed);
    if key >= MAX_KEYS {
        rtabort!("out of TLS keys (MAX_KEYS = {MAX_KEYS})");
    }
    KEY_DTORS[key].store(dtor.map_or(0, |d| d as usize), Relaxed);
    key
}

fn current_row() -> usize {
    let idx = unsafe { get_tid() } as usize + 1;
    if idx >= TID_LIMIT {
        rtabort!("out of TLS thread ids (TID_LIMIT = {TID_LIMIT})");
    }
    match ROW_INDEX[idx].load(Relaxed) {
        0 => alloc_row(idx),
        row => (row as usize - 1) * MAX_KEYS,
    }
}

fn alloc_row(idx: usize) -> usize {
    loop {
        let row = NEXT_ROW.fetch_add(1, Relaxed);
        if row >= MAX_ROWS {
            rtabort!("out of TLS rows (MAX_ROWS = {MAX_ROWS})");
        }
        if ROW_INDEX[idx].compare_exchange(0, (row + 1) as u32, Release, Relaxed).is_ok() {
            return row * MAX_KEYS;
        }
    }
}

#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    let row = current_row();
    ROWS[row + key].store(value.addr(), Release);
}

#[inline]
#[allow(implicit_provenance_casts)]
#[cfg(any(not(target_thread_local), test))]
pub unsafe fn get(key: Key) -> *mut u8 {
    let row = current_row();
    ROWS[row + key].load(Acquire) as *mut u8
}

#[inline]
pub unsafe fn destroy(key: Key) {
    KEY_DTORS[key].store(0, Relaxed);
    for idx in 0..ROW_INDEX.len() {
        match ROW_INDEX[idx].load(Relaxed) {
            0 => continue,
            row => ROWS[(row as usize - 1) * MAX_KEYS + key].store(0, Relaxed),
        }
    }
}
