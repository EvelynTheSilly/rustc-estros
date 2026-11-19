use crate::sys::{cap::Capability, syscalls};

/// TODO: doc
#[stable(feature = "estros", since = "1.0.0")]
pub struct Message {
    cid: u64,
    mid: u64,
}

impl Message {
    /// TODO: DOC
    #[stable(feature = "estros", since = "1.0.0")]
    pub unsafe fn from_raw(cap_id: u64, mid: u64) -> Self {
        Message { cid: cap_id, mid }
    }
    /// TODO: DOC
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn read_bytes(&self, buf: &mut [u8]) -> Option<usize> {
        let ret;
        unsafe { ret = syscalls::read_message(self.cid, self.mid, buf.as_mut_ptr(), buf.len()) }
        if (ret as i64) < 0 { None } else { Some(ret as usize) }
    }
    /// TODO: DOC
    #[stable(feature = "estros", since = "1.0.0")]
    pub fn read_caps(&self) -> Vec<Capability> {
        let ret;
        let mut idbuf = [0 as u64; 16];
        unsafe {
            ret = syscalls::read_message_caps(
                self.cid,
                self.mid,
                idbuf.as_mut_ptr() as *mut u8,
                idbuf.len(),
            )
        }

        if (ret as i64) < 0 {
            Vec::new()
        } else {
            let mut vec = Vec::with_capacity(ret as usize);
            unsafe {
                for i in 0..ret {
                    vec.push(Capability::from_id(idbuf[i as usize]));
                }
            }
            vec
        }
    }
}
