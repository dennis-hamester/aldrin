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

pub trait PacketizerExt: Packetizer {
    fn map_err<F, E>(self, f: F) -> MapError<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Error) -> E,
    {
        MapError {
            map_err: f,
            packetizer: self,
        }
    }
}

impl<T: Packetizer + ?Sized> PacketizerExt for T {}

#[derive(Debug)]
pub struct MapError<T: ?Sized, F> {
    map_err: F,
    packetizer: T,
}

impl<T, F, E> Packetizer for MapError<T, F>
where
    T: Packetizer + ?Sized,
    F: FnMut(T::Error) -> E,
{
    type Error = E;

    fn encode(&mut self, data: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.packetizer.encode(data, dst).map_err(&mut self.map_err)
    }

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Bytes>, Self::Error> {
        self.packetizer.decode(src).map_err(&mut self.map_err)
    }
}
