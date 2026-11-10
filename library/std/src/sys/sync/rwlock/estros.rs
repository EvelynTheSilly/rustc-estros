pub struct RwLock;

impl RwLock {
    #[inline]
    pub const fn new() -> RwLock {
        RwLock
    }

    #[inline]
    pub fn read(&self) {
        // TODO: safe rw lock
        //todo!("estros rwlock")
    }

    #[inline]
    pub fn try_read(&self) -> bool {
        // TODO: safe rw lock
        //todo!("estros rwlock")
        true
    }

    #[inline]
    pub fn write(&self) {
        // TODO: safe rw lock
        //todo!("estros rwlock")
    }

    #[inline]
    pub fn try_write(&self) -> bool {
        // TODO: safe rw lock
        // test change
        //todo!("estros rwlock")
        true
    }

    #[inline]
    pub unsafe fn read_unlock(&self) {
        // TODO: safe rw lock
        //todo!("estros rwlock")
    }

    #[inline]
    pub unsafe fn write_unlock(&self) {
        // TODO: safe rw lock
        //todo!("estros rwlock")
    }

    #[inline]
    pub unsafe fn downgrade(&self) {
        // TODO: safe rw lock
        //todo!("estros rwlock")
    }
}
