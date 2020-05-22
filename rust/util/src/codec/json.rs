#[cfg(test)]
mod test;

use super::Serializer;
use aldrin_proto::Message;
use bytes::buf::BufMutExt;
use bytes::{Bytes, BytesMut};

pub use serde_json::Error as JsonError;

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
    type Error = JsonError;

    fn serialize(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), JsonError> {
        if self.pretty {
            serde_json::to_writer_pretty(dst.writer(), &msg)
        } else {
            serde_json::to_writer(dst.writer(), &msg)
        }
    }

    fn deserialize(&mut self, src: Bytes) -> Result<Message, JsonError> {
        serde_json::from_slice(&src)
    }
}
