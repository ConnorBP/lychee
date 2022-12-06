// MIT License
//
// Copyright (c) 2018 frk <hazefrk+dev@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

mod clientclass;
mod netvars;
mod prop;
mod table;

use memflow::prelude::ModuleInfo;

pub use self::clientclass::ClientClassIterator;
pub use self::netvars::NetvarManager;
pub use self::table::RecvTable;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub base: usize,
    pub size: usize,
    pub data: Vec<u8>,
}

impl Module {
    // fn from_module_entry(me: &MODULEENTRY32W, name: &str, process: &Process) -> Option<Self> {
    //     let mut i = Module {
    //         name: name.to_string(),
    //         base: me.modBaseAddr as usize,
    //         size: me.modBaseSize as usize,
    //         data: vec![0u8; me.modBaseSize as usize],
    //     };

    //     if process.read_ptr(i.data.as_mut_ptr(), i.base, i.size) {
    //         return Some(i);
    //     }

    //     None
    // }
    pub fn from_memflow(mod_info: &ModuleInfo, mod_bytes: &Vec<u8>) -> Self {
        Self {
            name: mod_info.name.to_string(),
            base: mod_info.base.to_umem() as usize,
            size: mod_info.size as usize,
            data: mod_bytes.to_vec(),
        }
    }

    pub fn find_pattern(&self, pattern: &str) -> Option<usize> {
        crate::offsets::findpattern::find_pattern(&self.data, pattern)
    }

    /// o: Offset
    /// is_relative: Base has already been subtracted.
    pub fn get_raw<T: Copy>(&self, mut o: usize, is_relative: bool) -> Option<T> {
        if !is_relative {
            o -= self.base;
        }
        if o + std::mem::size_of::<T>() >= self.data.len() {
            return None;
        }
        let ptr = self.data.get(o)?;
        let raw: T = unsafe { std::mem::transmute_copy(ptr) };
        Some(raw)
    }

    /// is_relative: if true, the base has already been subtracted.
    pub fn get_slice(&self, mut offset: usize, len: usize, is_relative: bool) -> Option<&[u8]> {
        if !is_relative {
            offset = offset.wrapping_sub(self.base);
        }
        self.data.get(offset..(offset + len))
    }

    /// is_relative: if true, the base has already been subtracted.
    pub fn get(&self, mut offset: usize, is_relative: bool) -> Option<&[u8]> {
        if !is_relative {
            offset = offset.wrapping_sub(self.base);
        }
        self.data.get(offset..)
    }
}