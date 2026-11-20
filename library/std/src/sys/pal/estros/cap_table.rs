//! TODO: DOCS

use crate::collections::HashMap;
use crate::sync::{OnceLock, RwLock};
use crate::sys::cap::{Capability, message::Message};

/// TODO: DOCS
#[stable(feature = "estros", since = "1.0.0")]
pub type CapTable = HashMap<String, Capability>;

static CAP_TABLE: OnceLock<RwLock<CapTable>> = OnceLock::new();

fn cap_table() -> &'static RwLock<CapTable> {
    CAP_TABLE.get_or_init(|| RwLock::new(load_cap_table().unwrap_or_default()))
}

fn load_cap_table() -> Option<CapTable> {
    let cap = unsafe { Capability::from_id(0) };
    let message = cap.wait_message()?;
    parse_stream(&message)
}

/// TODO: DOCS
#[stable(feature = "estros", since = "1.0.0")]
pub fn get_cap(name: &str) -> Option<Capability> {
    cap_table().read().unwrap().get(name).cloned()
}

/// TODO: DOCS
#[stable(feature = "estros", since = "1.0.0")]
pub unsafe fn cap_set(name: &str, cap: Capability) -> Option<Capability> {
    cap_table().write().unwrap().insert(name.to_owned(), cap)
}

/// TODO: DOCS
#[stable(feature = "estros", since = "1.0.0")]
pub unsafe fn cap_unset(name: &str) -> Option<Capability> {
    cap_table().write().unwrap().remove(name)
}

enum ReadU64 {
    Value(u64),
    End,
    Malformed,
}

fn read_u64(message: &Message) -> ReadU64 {
    let mut buf = [0u8; 8];
    match message.read_bytes(&mut buf) {
        Some(8) => ReadU64::Value(u64::from_ne_bytes(buf)),
        Some(0) | None => ReadU64::End,
        Some(_) => ReadU64::Malformed,
    }
}

fn read_name(message: &Message, len: u64) -> Option<String> {
    let mut remaining = usize::try_from(len).ok()?;
    let mut bytes = Vec::new();
    let mut buf = [0u8; 64];
    while remaining > 0 {
        let want = remaining.min(buf.len());
        let read = message.read_bytes(&mut buf[..want])?;
        if read == 0 || read > want {
            return None;
        }
        bytes.extend_from_slice(&buf[..read]);
        remaining -= read;
    }

    let name = String::from_utf8(bytes).ok()?;
    name.is_ascii().then_some(name)
}

fn parse_stream(message: &Message) -> Option<CapTable> {
    let mut table = CapTable::new();
    loop {
        let id = match read_u64(message) {
            ReadU64::Value(id) => id,
            ReadU64::End => return Some(table),
            ReadU64::Malformed => return None,
        };
        let len = match read_u64(message) {
            ReadU64::Value(len) => len,
            ReadU64::End | ReadU64::Malformed => return None,
        };
        let name = read_name(message, len)?;
        table.insert(name, unsafe { Capability::from_id(id) });
    }
}
