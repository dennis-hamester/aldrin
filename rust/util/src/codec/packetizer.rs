use bytes::{Bytes, BytesMut};

pub trait Packetizer {
    type Error;

    fn encode(&mut self, data: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error>;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Bytes>, Self::Error>;
}
