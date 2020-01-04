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

use super::Broker;

const DEFAULT_FIFO_SIZE: usize = 16;

#[derive(Debug, Clone)]
pub struct Builder {
    fifo_size: usize,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> Broker {
        Broker::new(self.fifo_size)
    }

    pub fn set_fifo_size(mut self, fifo_size: usize) -> Self {
        self.fifo_size = fifo_size;
        self
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            fifo_size: DEFAULT_FIFO_SIZE,
        }
    }
}
