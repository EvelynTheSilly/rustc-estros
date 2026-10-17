use crate::alloc::Layout;

#[inline]
pub unsafe fn alloc(_layout: Layout) -> *mut u8 {
    todo!();
}

#[inline]
pub unsafe fn dealloc(_ptr: *mut u8, _layout: Layout) {
    todo!();
}

#[inline]
pub unsafe fn realloc(_ptr: *mut u8, _layout: Layout, _new_size: usize) -> *mut u8 {
    todo!();
}
