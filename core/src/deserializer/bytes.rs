use crate::buf_ext::ValueBufExt;
use crate::{DeserializeError, ValueKind};

#[derive(Debug)]
pub struct BytesDeserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    len: u32,
}

impl<'a, 'b> BytesDeserializer<'a, 'b> {
    pub(super) fn new(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Bytes)?;
        Self::new_without_value_kind(buf)
    }

    pub(super) fn new_without_value_kind(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;

        if buf.len() >= len as usize {
            Ok(Self { buf, len })
        } else {
            Err(DeserializeError::NoMoreElements)
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[u8] {
        &(*self.buf)[..self.len as usize]
    }

    pub fn advance(&mut self, cnt: usize) -> Result<(), DeserializeError> {
        if cnt <= self.len as usize {
            self.buf.try_skip(cnt)?;
            self.len -= cnt as u32;
            Ok(())
        } else {
            Err(DeserializeError::NoMoreElements)
        }
    }

    pub fn deserialize_extend<T>(self, bytes: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<u8>,
    {
        bytes.extend(self.as_slice().iter().copied());
        self.buf.try_skip(self.len as usize)
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        self.advance(self.len as usize)
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.finish_with(|| Ok(t))
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        if self.is_empty() {
            f()
        } else {
            Err(DeserializeError::MoreElementsRemain)
        }
    }

    pub fn skip_and_finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.skip_and_finish_with(|| Ok(t))
    }

    pub fn skip_and_finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        self.skip()?;
        f()
    }
}
