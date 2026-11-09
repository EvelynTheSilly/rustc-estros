use core::alloc::Layout;
use core::cmp;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::sys::pal::syscalls::map_page;

const PAGE_SIZE: usize = 4096;
const HEAP_BASE: usize = 0x1000000000;

const MAP_PAGE_WRITE: u64 = 1 << 1;

/// every block size is a multiple of this; also the header size and the
/// minimum payload alignment guaranteed without carving
const QUANTUM: usize = 32;
const ALLOC_BIT: usize = 1;
const SIZE_MASK: usize = !ALLOC_BIT;

/// header of every block; allocated blocks use `cap` (payload capacity) and
/// `back` (self address); free blocks reuse them as intrusive free-list links.
/// `back` is placed 8 bytes before the payload so `dealloc` can recover the
/// block start from the returned pointer alone, including carved blocks.
#[repr(C, align(32))]
struct Header {
    /// size of the whole block including this header; low bit = allocated
    si: usize,
    /// size_and_flags copy of the preceding block (0 at the heap base)
    prev_si: usize,
    /// allocated: payload capacity; free: next free block
    cap: usize,
    /// allocated: block start; free: previous free block
    back: usize,
}

/// bytes of [HEAP_BASE, HEAP_BASE + COMMITTED) currently backed by mapped pages
static COMMITTED: AtomicUsize = AtomicUsize::new(0);
/// allocator state lock; std Mutex/RwLock are unavailable (`todo!()`) so the
/// allocator carries its own spinlock
static LOCK: AtomicBool = AtomicBool::new(false);
/// end of the highest carved block, as an offset from HEAP_BASE; everything
/// in [HIGH_WATER, COMMITTED) is un-carved virgin space
static HIGH_WATER: AtomicUsize = AtomicUsize::new(0);
/// start offset of the last carved block; needed to set `prev_si` on fresh
/// blocks carved out of virgin space after a grow
static LAST: AtomicUsize = AtomicUsize::new(usize::MAX);

/// free list, sorted ascending by block address
static mut FREE_LIST: *mut Header = ptr::null_mut();

fn round_up_to_page(size: usize) -> usize {
    (size + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE
}

fn round_up_to_quantum(size: usize) -> Option<usize> {
    size.checked_add(QUANTUM - 1).map(|s| s & !(QUANTUM - 1))
}

fn align_up(value: usize, align: usize) -> Option<usize> {
    value.checked_add(align - 1).map(|v| v & !(align - 1))
}

unsafe fn header_at(start: usize) -> *mut Header {
    ptr::with_exposed_provenance_mut::<Header>(start)
}

/// acquires the allocator spinlock; released by dropping the returned guard
struct Guard;

impl Guard {
    fn acquire() -> Guard {
        while LOCK
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        Guard
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        LOCK.store(false, Ordering::Release);
    }
}

/// grows the allocation buffer by at least `extra` bytes, mapping whole pages
/// with the map_page syscall. all newly mapped pages are zeroed by the kernel
/// and are user-mode, read+write, non-executable.
///
/// returns Ok(end) where `end` is one past the last byte of the region now
/// committed ([HEAP_BASE, end)), or Err on mapping failure. on failure the
/// reservation is rolled back so a later attempt can retry.
///
/// must be called holding the allocator lock
fn grow(extra: usize) -> Result<usize, ()> {
    let extra = round_up_to_page(extra.max(1));

    // reserve the growth window; the highest observed writer owns the mapping
    // attempt, losers retry against the winner's result
    let mut committed = COMMITTED.load(Ordering::Acquire);
    loop {
        let new_committed = committed.checked_add(extra).ok_or(())?;
        match COMMITTED.compare_exchange(
            committed,
            new_committed,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => {
                if let Err(()) = map_pages(committed, new_committed) {
                    // roll back so the unbacked tail is not handed out
                    COMMITTED.fetch_sub(extra, Ordering::AcqRel);
                    return Err(());
                }
                return Ok(new_committed);
            }
            Err(actual) => committed = actual,
        }
    }
}

/// maps every page in [HEAP_BASE + committed_from, HEAP_BASE + committed_to)
/// sequentially. fails on the first error, leaving pages before it mapped;
/// the caller must roll back the atomic reservation.
///
/// concurrent growth: each block is contiguous, [prev_end, my_to); a loser that
/// retries has its window fully covered by the winner's larger region, so
/// interleaved reservations are safe: whoever wins maps its whole superset and
/// the region grows monotonically without holes.
fn map_pages(committed_from: usize, committed_to: usize) -> Result<(), ()> {
    let mut va = HEAP_BASE + committed_from;
    let end = HEAP_BASE + committed_to;
    while va < end {
        // RAII-style note: no unmap is possible, pages stay mapped for the
        // process lifetime; COMMITTED rollback above only rewinds the counter.
        let err = unsafe { map_page(va, MAP_PAGE_WRITE) };
        if err != 0 {
            return Err(());
        }
        va += PAGE_SIZE;
    }
    Ok(())
}

/// removes `h` from the free list (must be free and linked)
unsafe fn list_remove(h: *mut Header) {
    unsafe {
        let next = ptr::with_exposed_provenance_mut::<Header>((*h).cap);
        let prev = ptr::with_exposed_provenance_mut::<Header>((*h).back);
        if prev.is_null() {
            FREE_LIST = next;
        } else {
            (*prev).cap = next.expose_provenance();
        }
        if !next.is_null() {
            (*next).back = prev.expose_provenance();
        }
    }
}

/// inserts `h` into the free list keeping it sorted ascending by address
unsafe fn list_insert(h: *mut Header) {
    unsafe {
        let mut prev: *mut Header = ptr::null_mut();
        let mut cur = FREE_LIST;
        while !cur.is_null() && cur.addr() < h.addr() {
            prev = cur;
            cur = ptr::with_exposed_provenance_mut::<Header>((*cur).cap);
        }
        (*h).cap = cur.expose_provenance();
        (*h).back = prev.expose_provenance();
        if prev.is_null() {
            FREE_LIST = h;
        } else {
            (*prev).cap = h.expose_provenance();
        }
        if !cur.is_null() {
            (*cur).back = h.expose_provenance();
        }
    }
}

/// updates `prev_si` of the block following `block_end` if it is a carved
/// block; used after changing a block's size
unsafe fn update_following(block_end: usize, si: usize) {
    unsafe {
        let high_water = HEAP_BASE + HIGH_WATER.load(Ordering::Relaxed);
        if block_end < high_water {
            (*header_at(block_end)).prev_si = si;
        }
    }
}

/// core allocator; assumes size > 0. pointers returned via the public wrapper
unsafe fn alloc_locked(layout: Layout) -> *mut u8 {
    unsafe {
        let req_size = layout.size().max(1);
        let align = cmp::max(layout.align(), QUANTUM);

        let Some(payload_rq) = round_up_to_quantum(req_size) else { return ptr::null_mut() };
        // when carving is needed (align > QUANTUM) leave room for the worst
        // case front remainder (up to align - QUANTUM bytes) and the tail
        // header, so the payload always fits after alignment
        let Some(total) = (if align > QUANTUM {
            payload_rq
                .checked_add(align)
                .and_then(|t| t.checked_add(2 * QUANTUM))
        } else {
            payload_rq.checked_add(QUANTUM)
        }) else { return ptr::null_mut() };

        // search-or-grow loop; grow then retry the fit search
        loop {
            // first fit over the free list
            let mut cur = FREE_LIST;
            while !cur.is_null() {
                let bs = (*cur).si & SIZE_MASK;
                if bs >= total {
                    return alloc_from(cur.addr(), total, bs, align);
                }
                cur = ptr::with_exposed_provenance_mut::<Header>((*cur).cap);
            }

            // nothing fits: grow committed space and retry; the fresh region
            // becomes one free block the search will find
            match grow(total) {
                Err(()) => return ptr::null_mut(),
                Ok(new_committed) => {
                    let start = HEAP_BASE + HIGH_WATER.load(Ordering::Relaxed);
                    let end = HEAP_BASE + new_committed;
                    let last = LAST.load(Ordering::Relaxed);
                    let prev_si = if last == usize::MAX {
                        0
                    } else {
                        (*header_at(HEAP_BASE + last)).si
                    };
                    let h = header_at(start);
                    (*h).si = end - start;
                    (*h).prev_si = prev_si;
                    HIGH_WATER.store(new_committed, Ordering::Relaxed);
                    LAST.store(start - HEAP_BASE, Ordering::Relaxed);
                    list_insert(h);
                }
            }
        }
    }
}

/// carves `total` bytes out of the free block `a` (size `bs`) and allocates
/// from it; `align` is the rounded alignment for the payload. removes the
/// block from the free list, splits as needed and updates neighbors.
unsafe fn alloc_from(a: usize, total: usize, bs: usize, align: usize) -> *mut u8 {
    unsafe {
        let h = header_at(a);
        list_remove(h);
        let was_last = LAST.load(Ordering::Relaxed) == a - HEAP_BASE;
        let end = a + bs;

        if align <= QUANTUM {
            // simple case: payload starts right after the header
            if bs > total {
                // split tail into a free block
                let rem = header_at(a + total);
                (*rem).si = bs - total;
                (*rem).prev_si = total | ALLOC_BIT;
                list_insert(rem);
                if was_last {
                    LAST.store(a + total - HEAP_BASE, Ordering::Relaxed);
                }
            }
            (*h).si = total | ALLOC_BIT;
            (*h).cap = total - QUANTUM;
            (*h).back = a;
            update_following(end, (*h).si);
            // back slot stays self-referential; a + QUANTUM - 8 == a + 24
            return ptr::with_exposed_provenance_mut::<u8>(a + QUANTUM);
        }

        // carved case: front remainder [a, p - QUANTUM), tail [p - QUANTUM, end)
        let p = align_up(a + QUANTUM, align).unwrap();
        let front_rem = p - QUANTUM - a;
        // front_rem is always a multiple of QUANTUM for power-of-two align
        debug_assert!(front_rem == 0 || front_rem >= QUANTUM);
        let tail = p - QUANTUM;
        let t = header_at(tail);

        if front_rem >= QUANTUM {
            // shrink free block in place: same address, stays linked
            (*h).si = front_rem;
        }
        // the tail becomes the last carved block if we carved the last one
        if was_last {
            LAST.store(tail - HEAP_BASE, Ordering::Relaxed);
        }

        (*t).si = (end - tail) | ALLOC_BIT;
        (*t).prev_si = if front_rem >= QUANTUM { (*h).si } else { (*h).prev_si };
        (*t).cap = end - p;
        update_following(end, (*t).si);
        // self-reference doubles as `dealloc`'s back-pointer: p - 8 == tail + 24
        (*t).back = tail;
        ptr::with_exposed_provenance_mut::<u8>(p)
    }
}

unsafe fn dealloc_locked(p: *mut u8) {
    unsafe {
        // recover the block start from the back-pointer stored 8 bytes
        // before the payload (the tail header's `back` slot)
        let start = ptr::read(p.sub(8) as *const usize);
        let h = header_at(start);
        let si = (*h).si;
        let size = si & SIZE_MASK;
        let end = start + size;
        let high_water = HEAP_BASE + HIGH_WATER.load(Ordering::Relaxed);

        let mut cur_start = start;
        let mut cur_header = h;
        let mut cur_si = si & !ALLOC_BIT;
        let mut cur_prev_si = (*h).prev_si;

        // coalesce with the preceding free block
        let prev_si = (*h).prev_si;
        if prev_si != 0 && prev_si & ALLOC_BIT == 0 {
            let prev_start = start - (prev_si & SIZE_MASK);
            let prev = header_at(prev_start);
            list_remove(prev);
            cur_start = prev_start;
            cur_header = prev;
            cur_prev_si = (*prev).prev_si;
            cur_si = (*prev).si + size;
        }

        // coalesce with the following free block
        if end < high_water {
            let next = header_at(end);
            if (*next).si & ALLOC_BIT == 0 {
                list_remove(next);
                cur_si += (*next).si & SIZE_MASK;
            } else {
                (*next).prev_si = cur_si;
            }
        }

        (*cur_header).si = cur_si;
        (*cur_header).prev_si = cur_prev_si;
        list_insert(cur_header);
        let cur_end = cur_start + (cur_si & SIZE_MASK);
        update_following(cur_end, cur_si);
        if end == high_water {
            // the merged block is now the last carved block
            LAST.store(cur_start - HEAP_BASE, Ordering::Relaxed);
        }
    }
}

unsafe fn capacity_of(p: *mut u8) -> Option<usize> {
    unsafe {
        let start = ptr::read(p.sub(8) as *const usize);
        let h = header_at(start);
        if (*h).si & ALLOC_BIT == 0 {
            return None;
        }
        Some((*h).cap)
    }
}

#[inline]
pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    unsafe {
        let _guard = Guard::acquire();
        alloc_locked(layout)
    }
}

#[inline]
pub unsafe fn dealloc(ptr: *mut u8, _layout: Layout) {
    unsafe {
        let _guard = Guard::acquire();
        dealloc_locked(ptr);
    }
}

#[inline]
pub unsafe fn realloc(ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 {
    unsafe {
        let _guard = Guard::acquire();
        match capacity_of(ptr) {
            Some(cap) if new_size <= cap => {
                // fits in place; alignment is unchanged and contents are kept
                return ptr;
            }
            _ => {}
        }
        drop(_guard);
        let new_layout = match Layout::from_size_align(new_size, old_layout.align()) {
            Ok(l) => l,
            Err(_) => return ptr::null_mut(),
        };
        let new_ptr = {
            let _guard = Guard::acquire();
            alloc_locked(new_layout)
        };
        if !new_ptr.is_null() {
            let copy_len = usize::min(old_layout.size(), new_size);
            ptr::copy_nonoverlapping(ptr, new_ptr, copy_len);
            let _guard = Guard::acquire();
            dealloc_locked(ptr);
        }
        new_ptr
    }
}
