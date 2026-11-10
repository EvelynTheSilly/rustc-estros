use message::Message;

use crate::sys::syscalls::{
    self, lookup_mem_cap, send_message_to_receiver, spawn_capability, wait_on_message_from_receiver,
};

#[stable(feature = "estros", since = "1.0.0")]
pub mod message;

/// represents an os capability to... something? ¯\_(ツ)_/¯
/// TODO: DOCS
#[stable(feature = "estros", since = "1.0.0")]
#[derive(Clone)]
pub struct Capability {
    id: u64,
}

impl Capability {
    /// TODO: DOCS
    #[stable(feature = "estros", since = "1.0.0")]
    pub unsafe fn from_id(id: u64) -> Self {
        Capability { id }
    }
    /// TODO: DOCS
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn id(&self) -> u64 {
        self.id
    }
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn duplicate(&self) -> Capability {
        unsafe { Capability::from_id(syscalls::cap_duplicate(self.id) as u64) }
    }
    /// TODO: DOCS
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn from_mem(addr: usize, size: usize) -> Option<Self> {
        unsafe {
            let r = lookup_mem_cap(addr, size) as i64;
            if r < 0 {
                return None;
            }
            return Some(Capability::from_id(r as u64));
        }
    }
    /// TODO: DOCS
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn spawn_stream() -> Capability {
        Capability::spawn_internal(1)
    }
    /// TODO: DOCS
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn spawn_message() -> Capability {
        Capability::spawn_internal(2)
    }
    fn spawn_internal(kind: u64) -> Capability {
        let id;
        unsafe {
            id = spawn_capability(kind) as u64;
            return Capability::from_id(id);
        }
    }
    /// TODO: DOCS
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn wait_message(&self) -> Option<Message> {
        let mid;
        unsafe {
            mid = wait_on_message_from_receiver(self.id) as i64;
        };
        if mid < 0 {
            // error case
            return None;
        }
        unsafe { return Some(Message::from_raw(self.id, mid as u64)) }
    }
    /// TODO: DOCS
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn send_message(&self, data: &[u8], caps: [Capability]) -> Option<()> {
        let res;
        unsafe {
            res = send_message_to_receiver(
                self.id,
                data.as_ptr(),
                data.len(),
                caps.as_ptr() as *const u8,
                caps.len(),
            )
        };
        if res < 0 {
            // error case
            return None;
        }
        unsafe { return Some(()) }
    }
    /// returns bytes read
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn wait_stream(&self, bytes: &mut [u8]) -> usize {
        todo!()
    }
}
