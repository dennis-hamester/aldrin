use bytes::{Bytes, BytesMut};

pub trait Packetizer {
    type Error;

    fn encode(&mut self, data: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error>;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Bytes>, Self::Error>;
}

impl<T: Packetizer + ?Sized> Packetizer for &mut T {
    type Error = T::Error;

    fn encode(&mut self, data: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        (*self).encode(data, dst)
    }

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Bytes>, Self::Error> {
        (*self).decode(src)
    }
}

impl<T: Packetizer + ?Sized> Packetizer for Box<T> {
    type Error = T::Error;

    fn encode(&mut self, data: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        (**self).encode(data, dst)
    }

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Bytes>, Self::Error> {
        (**self).decode(src)
    }
}
