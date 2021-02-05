[![Build Status](https://circleci.com/gh/wbcchsyn/rust-mmap-allocator/tree/master.svg?style=shield)](https://circleci.com/gh/wbcchsyn/rust-mmap-allocator/cargo-readme/tree/master)
[![Build Status](https://travis-ci.org/wbcchsyn/rust-mmap-allocator.svg?branch=master)](https://travis-ci.org/wbcchsyn/rust-mmap-allocator)

# mmap-allocator

Declare struct `MmapAllocator` implementing `std::alloc::GlobalAlloc` whose backend is
'posix mmap()'.

This crate is only for 'unix' or 'linux' platform.

License: Apache-2.0 OR LGPL-3.0-or-later
