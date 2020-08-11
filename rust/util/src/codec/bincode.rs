#[cfg(test)]
mod test;

use super::{Endian, Serializer};
use aldrin_proto::Message;
use bincode::Options;
use bytes::buf::BufMutExt;
use bytes::{Bytes, BytesMut};

pub use bincode::Error as BincodeError;

#[derive(Debug)]
pub struct BincodeSerializer {
    endian: Endian,
}

impl BincodeSerializer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_endian(endian: Endian) -> Self {
        BincodeSerializer { endian }
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }
}

impl Default for BincodeSerializer {
    fn default() -> Self {
        BincodeSerializer {
            endian: Endian::Big,
        }
    }
}

impl Serializer for BincodeSerializer {
    type Error = BincodeError;

    fn serialize(&mut self, msg: Message) -> Result<BytesMut, BincodeError> {
        let mut dst = BytesMut::new();
        match self.endian {
            Endian::Big => bincode::options()
                .with_big_endian()
                .serialize_into((&mut dst).writer(), &msg)?,
            Endian::Little => bincode::options()
                .with_little_endian()
                .serialize_into((&mut dst).writer(), &msg)?,
        }
        Ok(dst)
    }

    fn deserialize(&mut self, src: Bytes) -> Result<Message, BincodeError> {
        match self.endian {
            Endian::Big => bincode::options().with_big_endian().deserialize(&src),
            Endian::Little => bincode::options().with_little_endian().deserialize(&src),
        }
    }
}
