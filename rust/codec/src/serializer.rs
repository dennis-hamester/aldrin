#[cfg(feature = "bincode")]
mod bincode;
#[cfg(feature = "json-serializer")]
mod json;

use aldrin_proto::Message;
use bytes::{Bytes, BytesMut};

#[cfg(feature = "bincode")]
pub use self::bincode::{Bincode, BincodeError};
#[cfg(feature = "json-serializer")]
pub use json::{Json, JsonError};

pub trait Serializer {
    type Error;

    fn serialize(&mut self, msg: Message) -> Result<BytesMut, Self::Error>;
    fn deserialize(&mut self, src: Bytes) -> Result<Message, Self::Error>;
}

impl<T: Serializer + ?Sized> Serializer for &mut T {
    type Error = T::Error;

    fn serialize(&mut self, msg: Message) -> Result<BytesMut, Self::Error> {
        (*self).serialize(msg)
    }

    fn deserialize(&mut self, src: Bytes) -> Result<Message, Self::Error> {
        (*self).deserialize(src)
    }
}

impl<T: Serializer + ?Sized> Serializer for Box<T> {
    type Error = T::Error;

    fn serialize(&mut self, msg: Message) -> Result<BytesMut, Self::Error> {
        (**self).serialize(msg)
    }

    fn deserialize(&mut self, src: Bytes) -> Result<Message, Self::Error> {
        (**self).deserialize(src)
    }
}

pub trait SerializerExt: Serializer {
    fn map_err<F, E>(self, f: F) -> MapError<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Error) -> E,
    {
        MapError {
            map_err: f,
            serializer: self,
        }
    }
}

impl<T: Serializer + ?Sized> SerializerExt for T {}

#[derive(Debug)]
pub struct MapError<T: ?Sized, F> {
    map_err: F,
    serializer: T,
}

impl<T, F, E> Serializer for MapError<T, F>
where
    T: Serializer + ?Sized,
    F: FnMut(T::Error) -> E,
{
    type Error = E;

    fn serialize(&mut self, msg: Message) -> Result<BytesMut, Self::Error> {
        self.serializer.serialize(msg).map_err(&mut self.map_err)
    }

    fn deserialize(&mut self, src: Bytes) -> Result<Message, Self::Error> {
        self.serializer.deserialize(src).map_err(&mut self.map_err)
    }
}
