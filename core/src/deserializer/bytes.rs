use crate::buf_ext::ValueBufExt;
use crate::{DeserializeError, ValueKind};

#[derive(Debug)]
pub enum BytesDeserializer<'a, 'b> {
    V1(Bytes1Deserializer<'a, 'b>),
    V2(Bytes2Deserializer<'a, 'b>),
}

impl<'a, 'b> BytesDeserializer<'a, 'b> {
    pub(super) fn new(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        match buf.try_get_discriminant_u8()? {
            ValueKind::Bytes1 => Bytes1Deserializer::new_without_value_kind(buf).map(Self::V1),
            ValueKind::Bytes2 => Bytes2Deserializer::new_without_value_kind(buf).map(Self::V2),
            _ => Err(DeserializeError::UnexpectedValue),
        }
    }

    pub fn as_slice(&self) -> Result<&[u8], DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.as_slice(),
            Self::V2(deserializer) => deserializer.as_slice(),
        }
    }

    pub fn advance(&mut self, cnt: usize) -> Result<(), DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.advance(cnt),
            Self::V2(deserializer) => deserializer.advance(cnt),
        }
    }

    pub fn deserialize_extend<T>(self, bytes: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<u8>,
    {
        match self {
            Self::V1(deserializer) => deserializer.deserialize_extend(bytes),
            Self::V2(deserializer) => deserializer.deserialize_extend(bytes),
        }
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.skip(),
            Self::V2(deserializer) => deserializer.skip(),
        }
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.finish(t),
            Self::V2(deserializer) => deserializer.finish(t),
        }
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        match self {
            Self::V1(deserializer) => deserializer.finish_with(f),
            Self::V2(deserializer) => deserializer.finish_with(f),
        }
    }

    pub fn skip_and_finish<T>(self, t: T) -> Result<T, DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.skip_and_finish(t),
            Self::V2(deserializer) => deserializer.skip_and_finish(t),
        }
    }

    pub fn skip_and_finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        match self {
            Self::V1(deserializer) => deserializer.skip_and_finish_with(f),
            Self::V2(deserializer) => deserializer.skip_and_finish_with(f),
        }
    }
}

#[derive(Debug)]
pub struct Bytes1Deserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    len: u32,
}

impl<'a, 'b> Bytes1Deserializer<'a, 'b> {
    pub(super) fn new(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Bytes1)?;
        Self::new_without_value_kind(buf)
    }

    pub(super) fn new_without_value_kind(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;

        if buf.len() >= len as usize {
            Ok(Self { buf, len })
        } else {
            Err(DeserializeError::InvalidSerialization)
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> Result<&[u8], DeserializeError> {
        Ok(&(*self.buf)[..self.len as usize])
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

    pub fn deserialize_extend<T>(mut self, bytes: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<u8>,
    {
        let slice = self.as_slice()?;
        bytes.extend(slice.iter().copied());
        self.advance(slice.len())
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

#[derive(Debug)]
pub struct Bytes2Deserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    len: usize,
}

impl<'a, 'b> Bytes2Deserializer<'a, 'b> {
    pub(super) fn new(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Bytes2)?;
        Self::new_without_value_kind(buf)
    }

    pub(super) fn new_without_value_kind(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()? as usize;
        Ok(Self { buf, len })
    }

    pub fn as_slice(&self) -> Result<&[u8], DeserializeError> {
        if self.buf.len() >= self.len {
            Ok(&(*self.buf)[..self.len])
        } else {
            Err(DeserializeError::InvalidSerialization)
        }
    }

    pub fn advance(&mut self, cnt: usize) -> Result<(), DeserializeError> {
        if self.len > 0 {
            if cnt <= self.len {
                self.buf.try_skip(cnt)?;
                self.len -= cnt;

                if self.len == 0 {
                    self.len = self.buf.try_get_varint_u32_le()? as usize;
                }

                Ok(())
            } else {
                Err(DeserializeError::NoMoreElements)
            }
        } else if cnt == 0 {
            Ok(())
        } else {
            Err(DeserializeError::NoMoreElements)
        }
    }

    pub fn deserialize_extend<T>(mut self, bytes: &mut T) -> Result<(), DeserializeError>
    where
        T: Extend<u8>,
    {
        loop {
            let slice = self.as_slice()?;

            if slice.is_empty() {
                break Ok(());
            }

            bytes.extend(slice.iter().copied());
            self.advance(slice.len())?;
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while self.len > 0 {
            self.advance(self.len)?;
        }

        Ok(())
    }

    pub fn finish<T>(self, t: T) -> Result<T, DeserializeError> {
        self.finish_with(|| Ok(t))
    }

    pub fn finish_with<T, F>(self, f: F) -> Result<T, DeserializeError>
    where
        F: FnOnce() -> Result<T, DeserializeError>,
    {
        if self.len == 0 {
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
