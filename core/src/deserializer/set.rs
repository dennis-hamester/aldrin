use crate::buf_ext::ValueBufExt;
use crate::tags::{KeyTag, KeyTagImpl};
use crate::{DeserializeError, DeserializeKey, ValueKind};
use std::marker::PhantomData;
use std::{fmt, iter};

#[derive(Debug)]
pub enum SetDeserializer<'a, 'b, K> {
    V1(Set1Deserializer<'a, 'b, K>),
    V2(Set2Deserializer<'a, 'b, K>),
}

impl<'a, 'b, K: KeyTag> SetDeserializer<'a, 'b, K> {
    pub(super) fn new(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        let kind = buf.try_get_discriminant_u8::<ValueKind>()?;

        if kind == K::Impl::VALUE_KIND_SET1 {
            Set1Deserializer::new_without_value_kind(buf).map(Self::V1)
        } else if kind == K::Impl::VALUE_KIND_SET2 {
            Set2Deserializer::new_without_value_kind(buf).map(Self::V2)
        } else {
            Err(DeserializeError::UnexpectedValue)
        }
    }

    pub fn deserialize<T: DeserializeKey<K>>(&mut self) -> Result<Option<T>, DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.deserialize(),
            Self::V2(deserializer) => deserializer.deserialize(),
        }
    }

    pub fn deserialize_extend<T, U>(self, set: &mut U) -> Result<(), DeserializeError>
    where
        T: DeserializeKey<K>,
        U: Extend<T>,
    {
        match self {
            Self::V1(deserializer) => deserializer.deserialize_extend(set),
            Self::V2(deserializer) => deserializer.deserialize_extend(set),
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

pub struct Set1Deserializer<'a, 'b, K> {
    buf: &'a mut &'b [u8],
    len: u32,
    _key: PhantomData<K>,
}

impl<'a, 'b, K: KeyTag> Set1Deserializer<'a, 'b, K> {
    pub(super) fn new(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(K::Impl::VALUE_KIND_SET1)?;
        Self::new_without_value_kind(buf)
    }

    pub(super) fn new_without_value_kind(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            len,
            _key: PhantomData,
        })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn deserialize<T: DeserializeKey<K>>(&mut self) -> Result<Option<T>, DeserializeError> {
        if self.is_empty() {
            Ok(None)
        } else {
            self.len -= 1;

            K::Impl::deserialize_key(self.buf)
                .and_then(T::try_from_key)
                .map(Some)
        }
    }

    pub fn deserialize_extend<T, U>(mut self, set: &mut U) -> Result<(), DeserializeError>
    where
        T: DeserializeKey<K>,
        U: Extend<T>,
    {
        while let Some(elem) = self.deserialize()? {
            set.extend(iter::once(elem));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.is_empty() {
            Ok(())
        } else {
            self.len -= 1;
            K::Impl::skip(self.buf)
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

impl<K> fmt::Debug for Set1Deserializer<'_, '_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Set1Deserializer");

        f.field("buf", &self.buf);
        f.field("len", &self.len);

        f.finish()
    }
}

pub struct Set2Deserializer<'a, 'b, K> {
    buf: &'a mut &'b [u8],
    empty: bool,
    _key: PhantomData<K>,
}

impl<'a, 'b, K: KeyTag> Set2Deserializer<'a, 'b, K> {
    pub(super) fn new(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(K::Impl::VALUE_KIND_SET2)?;
        Self::new_without_value_kind(buf)
    }

    pub(super) fn new_without_value_kind(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        Ok(Self {
            buf,
            empty: false,
            _key: PhantomData,
        })
    }

    pub fn deserialize<T: DeserializeKey<K>>(&mut self) -> Result<Option<T>, DeserializeError> {
        if self.empty {
            Ok(None)
        } else {
            match self.buf.try_get_discriminant_u8()? {
                ValueKind::None => {
                    self.empty = true;
                    Ok(None)
                }

                ValueKind::Some => K::Impl::deserialize_key(self.buf)
                    .and_then(T::try_from_key)
                    .map(Some),

                _ => Err(DeserializeError::InvalidSerialization),
            }
        }
    }

    pub fn deserialize_extend<T, U>(mut self, set: &mut U) -> Result<(), DeserializeError>
    where
        T: DeserializeKey<K>,
        U: Extend<T>,
    {
        while let Some(elem) = self.deserialize()? {
            set.extend(iter::once(elem));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if !self.empty {
            match self.buf.try_get_discriminant_u8()? {
                ValueKind::None => self.empty = true,
                ValueKind::Some => K::Impl::skip(self.buf)?,
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

impl<K> fmt::Debug for Set2Deserializer<'_, '_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Set2Deserializer");

        f.field("buf", &self.buf);
        f.field("empty", &self.empty);

        f.finish()
    }
}
