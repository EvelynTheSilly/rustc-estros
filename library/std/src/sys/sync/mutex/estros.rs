pub struct Mutex;

impl Mutex {
    #[inline]
    pub const fn new() -> Mutex {
        Mutex
    }

    #[inline]
    pub fn lock(&self) {
        todo!("estros mutex")
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        todo!("estros mutex")
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        todo!("estros mutex")
    }
}
