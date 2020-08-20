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
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_pretty(pretty: bool) -> Self {
        JsonSerializer { pretty }
    }

    pub fn pretty(&self) -> bool {
        self.pretty
    }
}

impl Default for JsonSerializer {
    fn default() -> Self {
        JsonSerializer { pretty: true }
    }
}

impl Serializer for JsonSerializer {
    type Error = JsonError;

    fn serialize(&mut self, msg: Message) -> Result<BytesMut, JsonError> {
        let mut dst = BytesMut::new();
        if self.pretty {
            serde_json::to_writer_pretty((&mut dst).writer(), &msg)?
        } else {
            serde_json::to_writer((&mut dst).writer(), &msg)?
        }
        Ok(dst)
    }

    fn deserialize(&mut self, src: Bytes) -> Result<Message, JsonError> {
        serde_json::from_slice(&src)
    }
}
