use crate::{SerializeError, MAX_VALUE_DEPTH};
use bytes::BytesMut;

#[derive(Debug)]
pub struct Serializer<'a> {
    buf: &'a mut BytesMut,
    depth: u8,
}

impl<'a> Serializer<'a> {
    pub(crate) fn new(buf: &'a mut BytesMut, depth: u8) -> Result<Self, SerializeError> {
        let mut this = Self { buf, depth };
        this.increment_depth()?;
        Ok(this)
    }

    fn increment_depth(&mut self) -> Result<(), SerializeError> {
        self.depth += 1;

        if self.depth <= MAX_VALUE_DEPTH {
            Ok(())
        } else {
            Err(SerializeError::TooDeeplyNested)
        }
    }
}
