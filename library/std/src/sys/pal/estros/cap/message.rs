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
    pub fn read(&self, buf: &mut [u8]) -> usize {
        todo!()
    }
}
