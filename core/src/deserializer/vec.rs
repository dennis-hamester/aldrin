use super::Deserializer;
use crate::buf_ext::ValueBufExt;
use crate::tags::Tag;
use crate::{Deserialize, DeserializeError, ValueKind};
use std::iter;

#[derive(Debug)]
pub enum VecDeserializer<'a, 'b> {
    V1(Vec1Deserializer<'a, 'b>),
    V2(Vec2Deserializer<'a, 'b>),
}

impl<'a, 'b> VecDeserializer<'a, 'b> {
    pub(super) fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        match buf.try_get_discriminant_u8()? {
            ValueKind::Vec1 => Vec1Deserializer::new_without_value_kind(buf, depth).map(Self::V1),
            ValueKind::Vec2 => Vec2Deserializer::new_without_value_kind(buf, depth).map(Self::V2),
            _ => Err(DeserializeError::UnexpectedValue),
        }
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(
        &mut self,
    ) -> Result<Option<U>, DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.deserialize(),
            Self::V2(deserializer) => deserializer.deserialize(),
        }
    }

    pub fn deserialize_extend<T, U, V>(self, vec: &mut V) -> Result<(), DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Extend<U>,
    {
        match self {
            Self::V1(deserializer) => deserializer.deserialize_extend(vec),
            Self::V2(deserializer) => deserializer.deserialize_extend(vec),
        }
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.skip_element(),
            Self::V2(deserializer) => deserializer.skip_element(),
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
pub struct Vec1Deserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    len: u32,
    depth: u8,
}

impl<'a, 'b> Vec1Deserializer<'a, 'b> {
    pub(super) fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Vec1)?;
        Self::new_without_value_kind(buf, depth)
    }

    pub(super) fn new_without_value_kind(
        buf: &'a mut &'b [u8],
        depth: u8,
    ) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;
        Ok(Self { buf, len, depth })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(
        &mut self,
    ) -> Result<Option<U>, DeserializeError> {
        if self.is_empty() {
            Ok(None)
        } else {
            self.len -= 1;
            let deserializer = Deserializer::new(self.buf, self.depth)?;
            deserializer.deserialize().map(Some)
        }
    }

    pub fn deserialize_extend<T, U, V>(mut self, vec: &mut V) -> Result<(), DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Extend<U>,
    {
        while let Some(elem) = self.deserialize()? {
            vec.extend(iter::once(elem));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.is_empty() {
            Ok(())
        } else {
            self.len -= 1;
            let deserializer = Deserializer::new(self.buf, self.depth)?;
            deserializer.skip()
        }
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while !self.is_empty() {
            self.skip_element()?;
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
pub struct Vec2Deserializer<'a, 'b> {
    buf: &'a mut &'b [u8],
    empty: bool,
    depth: u8,
}

impl<'a, 'b> Vec2Deserializer<'a, 'b> {
    pub(super) fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(ValueKind::Vec2)?;
        Self::new_without_value_kind(buf, depth)
    }

    pub(super) fn new_without_value_kind(
        buf: &'a mut &'b [u8],
        depth: u8,
    ) -> Result<Self, DeserializeError> {
        Ok(Self {
            buf,
            empty: false,
            depth,
        })
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(
        &mut self,
    ) -> Result<Option<U>, DeserializeError> {
        if self.empty {
            Ok(None)
        } else {
            match self.buf.try_get_discriminant_u8()? {
                ValueKind::None => {
                    self.empty = true;
                    Ok(None)
                }

                ValueKind::Some => {
                    let deserializer = Deserializer::new(self.buf, self.depth)?;
                    deserializer.deserialize().map(Some)
                }

                _ => Err(DeserializeError::InvalidSerialization),
            }
        }
    }

    pub fn deserialize_extend<T, U, V>(mut self, vec: &mut V) -> Result<(), DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        V: Extend<U>,
    {
        while let Some(elem) = self.deserialize()? {
            vec.extend(iter::once(elem));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if !self.empty {
            match self.buf.try_get_discriminant_u8()? {
                ValueKind::None => self.empty = true,
                ValueKind::Some => Deserializer::new(self.buf, self.depth)?.skip()?,
                _ => return Err(DeserializeError::InvalidSerialization),
            }
        }

        Ok(())
    }

    pub fn skip(mut self) -> Result<(), DeserializeError> {
        while !self.empty {
            self.skip_element()?;
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
        if self.empty {
            f()
        } else {
            match self.buf.try_get_discriminant_u8()? {
                ValueKind::None => f(),
                ValueKind::Some => Err(DeserializeError::MoreElementsRemain),
                _ => Err(DeserializeError::InvalidSerialization),
            }
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
