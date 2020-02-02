// Copyright (c) 2020 Dennis Hamester <dennis.hamester@gmail.com>
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

use super::Serializer;
use aldrin_proto::Message;
use bytes::buf::BufMutExt;
use bytes::{Bytes, BytesMut};
use serde_json::Error;

#[derive(Debug)]
pub struct JsonSerializer {
    pretty: bool,
}

impl JsonSerializer {
    pub fn new(pretty: bool) -> Self {
        JsonSerializer { pretty }
    }

    pub fn pretty(&self) -> bool {
        self.pretty
    }
}

impl Serializer for JsonSerializer {
    type Error = Error;

    fn serialize(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if self.pretty {
            serde_json::to_writer_pretty(dst.writer(), &msg)
        } else {
            serde_json::to_writer(dst.writer(), &msg)
        }
    }

    fn deserialize(&mut self, src: Bytes) -> Result<Message, Self::Error> {
        serde_json::from_slice(&src)
    }
}
