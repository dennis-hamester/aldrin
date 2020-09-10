#[cfg(test)]
mod test;

use super::Serializer;
use crate::Endian;
use aldrin_proto::Message;
use bincode::Options;
use bytes::buf::BufMutExt;
use bytes::{Bytes, BytesMut};

pub use bincode::Error as BincodeError;

#[derive(Debug)]
pub struct Bincode {
    endian: Endian,
}

impl Bincode {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_endian(endian: Endian) -> Self {
        Bincode { endian }
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }
}

impl Default for Bincode {
    fn default() -> Self {
        Bincode {
            endian: Endian::Big,
        }
    }
}

impl Serializer for Bincode {
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
