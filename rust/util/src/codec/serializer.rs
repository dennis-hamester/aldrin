use aldrin_proto::Message;
use bytes::{Bytes, BytesMut};

pub trait Serializer {
    type Error;

    fn serialize(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), Self::Error>;
    fn deserialize(&mut self, src: Bytes) -> Result<Message, Self::Error>;
}

impl<T: Serializer + ?Sized> Serializer for &mut T {
    type Error = T::Error;

    fn serialize(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        (*self).serialize(msg, dst)
    }

    fn deserialize(&mut self, src: Bytes) -> Result<Message, Self::Error> {
        (*self).deserialize(src)
    }
}

impl<T: Serializer + ?Sized> Serializer for Box<T> {
    type Error = T::Error;

    fn serialize(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).serialize(msg, dst)
    }

    fn deserialize(&mut self, src: Bytes) -> Result<Message, Self::Error> {
        (**self).deserialize(src)
    }
}
