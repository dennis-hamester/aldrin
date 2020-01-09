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

use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct Service {
    function_calls: HashSet<u32>,
}

impl Service {
    pub fn new() -> Self {
        Service {
            function_calls: HashSet::new(),
        }
    }

    pub fn add_function_call(&mut self, serial: u32) {
        let unique = self.function_calls.insert(serial);
        debug_assert!(unique);
    }

    pub fn function_calls<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        self.function_calls.iter().cloned()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ServiceUuid(pub Uuid);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ServiceCookie(pub Uuid);
