pub type Key = usize;

pub fn create(_dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    todo!("estros tls key")
}

#[inline]
pub unsafe fn set(_key: Key, _value: *mut u8) {
    todo!("estros tls key")
}

#[inline]
#[cfg(any(not(target_thread_local), test))]
pub unsafe fn get(_key: Key) -> *mut u8 {
    todo!("estros tls key")
}

#[inline]
pub unsafe fn destroy(_key: Key) {
    todo!("estros tls key")
}
