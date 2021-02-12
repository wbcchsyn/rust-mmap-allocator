[![Build Status](https://circleci.com/gh/wbcchsyn/rust-mmap-allocator/tree/master.svg?style=shield)](https://circleci.com/gh/wbcchsyn/rust-mmap-allocator/cargo-readme/tree/master)
[![Build Status](https://travis-ci.org/wbcchsyn/rust-mmap-allocator.svg?branch=master)](https://travis-ci.org/wbcchsyn/rust-mmap-allocator)

# mmap-allocator

`mmap-allocator` declares struct `MmapAllocator` and function 'page\_size' for 'unix' or 'linux' platform.

`MmapAllocator` implements `std::alloc::GlobalAlloc` whose backend is 'posix mmap'.

'page\_size' returns OS page size.
('unix' and 'linux' os pass memory to a process by multipile of page size; if a process
requires 32 bytes heap memory and if the OS page size is 4096 bytes, OS passes 4096 bytes
memory chunk. Usually 'malloc' deals it to use memory effectively.)

License: Apache-2.0 OR LGPL-3.0-or-later
