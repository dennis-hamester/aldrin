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
    pub fn new(endian: Endian) -> Self {
        BincodeSerializer { endian }
    }
}

impl Serializer for BincodeSerializer {
    type Error = BincodeError;

    fn serialize(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), BincodeError> {
        match self.endian {
            Endian::Big => bincode::options()
                .with_fixint_encoding()
                .with_big_endian()
                .serialize_into(dst.writer(), &msg),
            Endian::Little => bincode::options()
                .with_fixint_encoding()
                .with_little_endian()
                .serialize_into(dst.writer(), &msg),
        }
    }

    fn deserialize(&mut self, src: Bytes) -> Result<Message, BincodeError> {
        match self.endian {
            Endian::Big => bincode::options()
                .with_fixint_encoding()
                .with_big_endian()
                .deserialize(&src),
            Endian::Little => bincode::options()
                .with_fixint_encoding()
                .with_little_endian()
                .deserialize(&src),
        }
    }
}
