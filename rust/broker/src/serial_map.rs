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

use std::convert::TryFrom;

#[derive(Debug)]
pub(crate) struct SerialMap<T> {
    next: u32,
    free: Vec<u32>,
    len: usize,
    objs: Vec<Option<T>>,
}

impl<T> SerialMap<T> {
    pub fn new() -> Self {
        SerialMap {
            next: 0,
            free: Vec::new(),
            len: 0,
            objs: Vec::new(),
        }
    }

    pub fn insert(&mut self, obj: T) -> u32 {
        match self.free.pop() {
            Some(serial) => {
                debug_assert!(self.objs[serial as usize].is_none());
                self.objs[serial as usize] = Some(obj);
                self.len += 1;
                serial
            }

            None => {
                let serial = self.next;
                let serial_usize = usize::try_from(serial).expect("too many serials");
                self.next = self.next.checked_add(1).expect("too many serials");
                self.objs.push(Some(obj));
                self.len += 1;
                debug_assert_eq!(self.objs.len() - 1, serial_usize);
                serial
            }
        }
    }

    pub fn remove(&mut self, serial: u32) -> Option<T> {
        debug_assert!(self.objs.len() > serial as usize);

        match self.objs[serial as usize].take() {
            Some(obj) => {
                debug_assert!(self.next > serial);
                debug_assert!(self.len > 0);

                self.len -= 1;
                if self.len == 0 {
                    self.next = 0;
                    self.objs.clear();
                } else if serial == (self.next - 1) {
                    self.next -= 1;
                    self.objs.pop();
                } else {
                    debug_assert!(!self.free.contains(&serial));
                    self.free.push(serial);
                }

                Some(obj)
            }

            None => None,
        }
    }
}
