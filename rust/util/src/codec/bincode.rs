#[cfg(test)]
mod test;

use super::{Endian, Serializer};
use aldrin_proto::Message;
use bincode::Config;
use bytes::buf::BufMutExt;
use bytes::{Bytes, BytesMut};
use std::fmt;

pub use bincode::Error as BincodeError;

pub struct BincodeSerializer(Config);

impl BincodeSerializer {
    pub fn new(endian: Endian) -> Self {
        let mut config = bincode::config();
        match endian {
            Endian::Big => config.big_endian(),
            Endian::Little => config.little_endian(),
        };
        BincodeSerializer(config)
    }
}

impl fmt::Debug for BincodeSerializer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("BincodeSerializer")
            .field(&format_args!("_"))
            .finish()
    }
}

impl Serializer for BincodeSerializer {
    type Error = BincodeError;

    fn serialize(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), BincodeError> {
        self.0.serialize_into(dst.writer(), &msg)
    }

    fn deserialize(&mut self, src: Bytes) -> Result<Message, BincodeError> {
        self.0.deserialize(&src)
    }
}
