//! The capability name table handed to a process at spawn.
//!
//! TODO: DOCS

use crate::collections::HashMap;
use crate::sync::OnceLock;
use crate::sys::cap::{Capability, message::Message};

/// A mapping from a capability's name to the capability itself.
///
/// TODO: DOCS
#[stable(feature = "estros", since = "1.0.0")]
pub type CapTable = HashMap<String, Capability>;

static CAP_TABLE: OnceLock<CapTable> = OnceLock::new();

/// Returns the capability table for this process.
///
/// The table is read from capability 0 the first time this is called.
///
/// TODO: DOCS
#[stable(feature = "estros", since = "1.0.0")]
pub fn get_cap_table() -> &'static CapTable {
    CAP_TABLE.get_or_init(load_cap_table)
}

fn load_cap_table() -> CapTable {
    // Capability 0 is handed to every process when it is spawned.
    let cap = unsafe { Capability::from_id(0) };
    let Some(message) = cap.wait_message() else {
        return CapTable::new();
    };
    // A malformed stream leaves this process without a table rather than with
    // a partially populated one.
    parse_stream(&message).unwrap_or_default()
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
        // Messages with attached capabilities outlive their data, so an empty
        // read can also mean the end of the stream.
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

/// Parses `cap_id (u64), len (u64), name (len bytes)` records until the
/// message is exhausted. Returns `None` if any record is malformed.
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
            // The message ended partway through a header.
            ReadU64::End | ReadU64::Malformed => return None,
        };
        let name = read_name(message, len)?;
        table.insert(name, unsafe { Capability::from_id(id) });
    }
}
