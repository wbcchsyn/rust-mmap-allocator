use libc::{off_t, size_t};
use std::alloc::{GlobalAlloc, Layout};
use std::os::raw::{c_int, c_void};
use std::ptr;

/// Allocator whose backend is mmap(2)
#[derive(Debug, Clone, Copy)]
pub struct MmapAllocator;

impl Default for MmapAllocator {
    fn default() -> Self {
        Self
    }
}

/// # Portability
///
/// alloc() calls mmap() with flag MAP_ANONYMOUS.
/// Many systems support the flag, however, it is not specified in POSIX.
///
/// # Safety
///
/// All functions are thread safe.
///
/// # Error
///
/// Each function don't cause panic but set OS errno on error.
///
/// Note that it is not an error to deallocate pointer which is not allocated.
/// This is the spec of munmap(2). See `man 2 munmap` for details.
unsafe impl GlobalAlloc for MmapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        const ADDR: *mut c_void = ptr::null_mut::<c_void>();
        let length = layout.size() as size_t;
        const PROT: c_int = libc::PROT_READ | libc::PROT_WRITE;

        // No backend file.
        // MAP_UNINITIALIZED is not very common.
        // To make this module portable, don't use it.
        const FLAGS: c_int = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS;
        const FD: c_int = -1; // Should be -1 if flags includes MAP_ANONYMOUS. See `man 2 mmap`
        const OFFSET: off_t = 0; // Should be 0 if flags includes MAP_ANONYMOUS. See `man 2 mmap`

        match mmap(ADDR, length, PROT, FLAGS, FD, OFFSET) {
            libc::MAP_FAILED => ptr::null_mut::<u8>(),
            ret => ret as *mut u8,
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let addr = ptr as *mut c_void;
        let length = layout.size() as size_t;

        munmap(addr, length);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // alloc() calls mmap() with the flags which always fills the memory 0.
        self.alloc(layout)
    }
}

extern "C" {
    fn mmap(
        addr: *mut c_void,
        length: size_t,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: off_t,
    ) -> *mut c_void;

    fn munmap(addr: *mut c_void, length: size_t);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;
    use std::ptr;

    const ENOERR: i32 = 0;

    fn clear_errno() {
        unsafe { *libc::__errno_location() = 0 }
    }

    fn errno() -> i32 {
        unsafe { *libc::__errno_location() }
    }

    #[test]
    fn default() {
        let _alloc = MmapAllocator::default();
    }

    #[test]
    fn allocate() {
        unsafe {
            type T = i64;
            let alloc = MmapAllocator::default();

            let layout = Layout::new::<i64>();
            let ptr = alloc.alloc(layout) as *mut T;
            assert_ne!(std::ptr::null(), ptr);

            *ptr = 84;
            assert_eq!(84, *ptr);

            *ptr = *ptr * -2;
            assert_eq!(-168, *ptr);

            alloc.dealloc(ptr as *mut u8, layout)
        }
    }

    #[test]
    fn allocate_too_large() {
        unsafe {
            clear_errno();

            type T = String;
            let alloc = MmapAllocator::default();

            let align = mem::align_of::<T>();
            let size = std::usize::MAX - mem::size_of::<T>();
            let layout = Layout::from_size_align(size, align).unwrap();

            assert_eq!(ptr::null(), alloc.alloc(layout));
            assert_ne!(ENOERR, errno());
        }
    }

    #[test]
    fn allocate_zero_size() {
        unsafe {
            clear_errno();

            type T = String;
            let alloc = MmapAllocator::default();

            let align = mem::align_of::<T>();
            let size = 0;
            let layout = Layout::from_size_align(size, align).unwrap();

            assert_eq!(ptr::null(), alloc.alloc(layout));
            assert_ne!(ENOERR, errno());
        }
    }

    #[test]
    fn alloc_zeroed() {
        unsafe {
            type T = [u8; 1025];
            let alloc = MmapAllocator::default();

            let layout = Layout::new::<T>();
            let ptr = alloc.alloc_zeroed(layout) as *const T;
            let s: &[u8] = &*ptr;

            for u in s {
                assert_eq!(0, *u);
            }
        }
    }
}
