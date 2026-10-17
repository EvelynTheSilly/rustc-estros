pub struct RwLock;

impl RwLock {
    #[inline]
    pub const fn new() -> RwLock {
        RwLock
    }

    #[inline]
    pub fn read(&self) {
        todo!("estros rwlock")
    }

    #[inline]
    pub fn try_read(&self) -> bool {
        todo!("estros rwlock")
    }

    #[inline]
    pub fn write(&self) {
        todo!("estros rwlock")
    }

    #[inline]
    pub fn try_write(&self) -> bool {
        todo!("estros rwlock")
    }

    #[inline]
    pub unsafe fn read_unlock(&self) {
        todo!("estros rwlock")
    }

    #[inline]
    pub unsafe fn write_unlock(&self) {
        todo!("estros rwlock")
    }

    #[inline]
    pub unsafe fn downgrade(&self) {
        todo!("estros rwlock")
    }
}
