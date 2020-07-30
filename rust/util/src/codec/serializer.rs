use aldrin_proto::Message;
use bytes::{Bytes, BytesMut};

pub trait Serializer {
    type Error;

    fn serialize(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), Self::Error>;
    fn deserialize(&mut self, src: Bytes) -> Result<Message, Self::Error>;
}
