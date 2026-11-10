pub struct Mutex;

impl Mutex {
    #[inline]
    pub const fn new() -> Mutex {
        Mutex
    }

    #[inline]
    pub fn lock(&self) {
        // No-op, very unsafe but we ball
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        true
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        // No-op, very unsafe but we ball
    }
}
