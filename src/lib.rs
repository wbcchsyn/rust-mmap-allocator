// Copyright 2020,2021 Shin Yoshida
//
// This is part of rust-mmap-allocator
//
// "LGPL-3.0-or-later OR Apache-2.0"
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

//! `mmap-allocator` declares struct `MmapAllocator` and function 'page\_size' for 'unix' or 'linux' platform.
//!
//! `MmapAllocator` implements `std::alloc::GlobalAlloc` whose backend is 'posix mmap'.
//!
//! 'page\_size' returns OS page size.
//! ('unix' and 'linux' os pass memory to a process by multipile of page size; if a process
//! requires 32 bytes heap memory and if the OS page size is 4096 bytes, OS passes 4096 bytes
//! memory chunk. Usually 'malloc' deals it to use memory effectively.)

#[cfg(unix)]
mod mmap_allocator;

#[cfg(unix)]
pub use crate::mmap_allocator::{page_size, MmapAllocator};
