use libc::{off_t, size_t};
use std::alloc::{GlobalAlloc, Layout, System};
use std::os::raw::{c_int, c_void};

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
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
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
            type T = String;
            let alloc = MmapAllocator::default();

            let align = mem::align_of::<T>();
            let size = std::usize::MAX - mem::size_of::<T>();
            let layout = Layout::from_size_align(size, align).unwrap();

            assert_eq!(ptr::null(), alloc.alloc(layout));
        }
    }
}
