// Copyright 2020,2021 Shin Yoshida
//
// "LGPL-3.0-or-later OR Apache-2.0"
//
// This is part of rust-mmap-allocator
//
//  rust-mmap-allocator is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  rust-mmap-allocator is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with rust-mmap-allocator.  If not, see <http://www.gnu.org/licenses/>.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use libc::{off_t, size_t};
use std::alloc::{GlobalAlloc, Layout};
use std::cell::Cell;
use std::os::raw::{c_int, c_long, c_void};
use std::ptr;

/// Implementation of std::alloc::GlobalAlloc whose backend is mmap(2)
#[derive(Debug, Clone, Copy)]
pub struct MmapAllocator;

impl Default for MmapAllocator {
    #[inline]
    fn default() -> Self {
        Self
    }
}

impl MmapAllocator {
    /// Creates a new instance.
    #[inline]
    pub const fn new() -> Self {
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
    /// # Panics
    ///
    /// This method can panic if the align of `layout` is greater than the kernel page align.
    #[inline]
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
            ret => {
                let ptr = ret as usize;
                assert_eq!(0, ptr % layout.align());
                ret as *mut u8
            }
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let addr = ptr as *mut c_void;
        let length = layout.size() as size_t;

        munmap(addr, length);
    }

    /// # Panics
    ///
    /// This method can panic if the align of `layout` is greater than the kernel page align.
    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // alloc() calls mmap() with the flags which always fills the memory 0.
        self.alloc(layout)
    }

    #[cfg(linux)]
    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_address = ptr as *mut c_void;
        let old_size = layout.size() as size_t;
        let new_size = new_size as size_t;
        let FLAGS = libc::MREMAP_MAYMOVE;

        match mremap(old_address, old_size, new_size, FLAGS) {
            libc::MAP_FAILED => ptr::null_mut::<u8>(),
            ret => ret as *mut u8,
        }
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

    #[cfg(linux)]
    fn mremap(
        old_address: *mut c_void,
        old_size: size_t,
        new_size: size_t,
        flags: c_int,
    ) -> *mut c_void;
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
            let ptr = alloc.alloc_zeroed(layout) as *mut T;
            let s: &[u8] = &*ptr;

            for u in s {
                assert_eq!(0, *u);
            }

            alloc.dealloc(ptr as *mut u8, layout);
        }
    }

    #[test]
    fn realloc() {
        unsafe {
            type T = [u8; 1025];
            let alloc = MmapAllocator::default();

            let layout = Layout::new::<T>();
            let ptr = alloc.alloc(layout) as *mut T;

            let ts = &mut *ptr;
            for t in ts.iter_mut() {
                *t = 1;
            }

            type U = (T, T);

            let new_size = mem::size_of::<U>();
            let ptr = alloc.realloc(ptr as *mut u8, layout, new_size) as *mut T;
            let layout = Layout::from_size_align(new_size, layout.align()).unwrap();

            let ts = &mut *ptr;
            for t in ts.iter_mut() {
                assert_eq!(1, *t);
                *t = 2;
            }

            let new_size = mem::size_of::<u8>();
            let ptr = alloc.realloc(ptr as *mut u8, layout, new_size);
            let layout = Layout::from_size_align(new_size, layout.align()).unwrap();

            assert_eq!(2, *ptr);

            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn realloc_too_large() {
        unsafe {
            type T = [u8; 1025];
            let alloc = MmapAllocator::default();

            let layout = Layout::new::<T>();
            let ptr = alloc.alloc(layout) as *mut T;

            let ts = &mut *ptr;
            for t in ts.iter_mut() {
                *t = 1;
            }

            let new_size = std::usize::MAX - mem::size_of::<T>();
            let new_ptr = alloc.realloc(ptr as *mut u8, layout, new_size);

            assert!(new_ptr.is_null());
            assert_ne!(ENOERR, errno());

            for t in ts.iter() {
                assert_eq!(1, *t);
            }

            alloc.dealloc(ptr as *mut u8, layout);
        }
    }
}

thread_local! {
    static PAGE_SIZE: Cell<usize> = Cell::new(0);
}

/// Returns OS Page Size.
///
/// See crate document for details.
#[inline]
pub fn page_size() -> usize {
    PAGE_SIZE.with(|s| match s.get() {
        0 => {
            let ret = unsafe { sysconf(libc::_SC_PAGE_SIZE) as usize };
            s.set(ret);
            ret
        }
        ret => ret,
    })
}

extern "C" {
    fn sysconf(name: c_int) -> c_long;
}
