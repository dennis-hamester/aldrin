// Copyright (c) 2019 Dennis Hamester <dennis.hamester@gmail.com>
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

use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct SerialMap<T> {
    elems: HashMap<u32, T>,
    next: u32,
}

impl<T> SerialMap<T> {
    pub fn new() -> Self {
        SerialMap {
            next: 0,
            elems: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }

    pub fn insert(&mut self, obj: T) -> u32 {
        let serial = self.next;
        self.next = self.next.wrapping_add(1);
        let dup = self.elems.insert(serial, obj);
        assert!(dup.is_none());
        serial
    }

    pub fn remove(&mut self, serial: u32) -> Option<T> {
        self.elems.remove(&serial)
    }

    pub fn get_mut(&mut self, serial: u32) -> Option<&mut T> {
        self.elems.get_mut(&serial)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u32, &mut T)> {
        self.elems.iter_mut().map(|(&s, e)| (s, e))
    }
}
