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

#![allow(clippy::double_parens)]

use log::debug;
use memflow::prelude::ModuleInfo;

use std::collections::BTreeMap;

use crate::offsets::games::csgo::Module;

#[derive(Debug, Clone, PartialEq)]
pub struct NetvarManager {
    tables: BTreeMap<String, super::RecvTable>,
}

impl NetvarManager {
    pub fn new(first: usize, client_module: &ModuleInfo, client_bytes: &Vec<u8>) -> Option<Self> {
        let module = Module::from_memflow(client_module, client_bytes);
        debug!("First ClientClass at {:#X}", first);

        let classes = super::ClientClassIterator::new(first + module.base, &module);
        let tables = classes
            .map(|c| (c.table.name.clone(), c.table))
            .collect::<BTreeMap<_, _>>();
        debug!("Added {} parent RecvTables!", tables.len());
        Some(NetvarManager { tables })
    }

    pub fn get_offset(&self, table_name: &str, netvar_name: &str) -> Option<i32> {
        self.tables.get(table_name)?.get_offset(netvar_name)
    }
}
