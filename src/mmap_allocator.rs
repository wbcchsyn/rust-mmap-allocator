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

unsafe impl GlobalAlloc for MmapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let addr = ptr::null_mut::<c_void>();
        let length = layout.size() as size_t;
        let prot = libc::PROT_READ | libc::PROT_WRITE;

        // MAP_UNINITIALIZED is not very common.
        // To make this module portable, don't use it.
        let flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS; // No backend file.
        let fd: c_int = -1; // Should be -1 if flags == MAP_ANONYMOUS. See `man 2 mmap`
        let offset: off_t = 0; // Should be 0 if flags == MAP_ANONYMOUS. See `man 2 mmap`

        match mmap(addr, length, prot, flags, fd, offset) {
            libc::MAP_FAILED => ptr::null_mut::<u8>(),
            ret => ret as *mut u8,
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let addr = ptr as *mut c_void;
        let length = layout.size() as size_t;

        munmap(addr, length);
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
}
