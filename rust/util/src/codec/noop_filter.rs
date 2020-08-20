use super::Filter;
use bytes::BytesMut;
use std::convert::Infallible;

#[derive(Default, Debug)]
pub struct NoopFilter;

impl Filter for NoopFilter {
    type Error = Infallible;

    fn forward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error> {
        Ok(data)
    }

    fn backward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error> {
        Ok(data)
    }
}
