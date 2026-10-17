use crate::sys::sync::Mutex;
use crate::time::Duration;

pub struct Condvar;

impl Condvar {
    #[inline]
    pub const fn new() -> Condvar {
        Condvar
    }

    #[inline]
    pub fn notify_one(&self) {}

    #[inline]
    pub fn notify_all(&self) {}

    pub unsafe fn wait(&self, _mutex: &Mutex) {
        todo!("estros condvar")
    }

    pub unsafe fn wait_timeout(&self, _mutex: &Mutex, _dur: Duration) -> bool {
        todo!("estros condvar")
    }
}
