use crate::buf_ext::ValueBufExt;
use crate::tags::{KeyTag, KeyTagImpl};
use crate::{DeserializeError, DeserializeKey};
use std::marker::PhantomData;
use std::{fmt, iter};

pub struct SetDeserializer<'a, 'b, K> {
    buf: &'a mut &'b [u8],
    len: u32,
    _key: PhantomData<K>,
}

impl<'a, 'b, K: KeyTag> SetDeserializer<'a, 'b, K> {
    pub(super) fn new(buf: &'a mut &'b [u8]) -> Result<Self, DeserializeError> {
        K::Impl::deserialize_set_value_kind(buf)?;
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

    pub fn deserialize<T: DeserializeKey<K>>(&mut self) -> Result<T, DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            K::Impl::deserialize_key(self.buf).and_then(T::try_from_key)
        }
    }

    pub fn deserialize_extend<T, U>(mut self, set: &mut U) -> Result<(), DeserializeError>
    where
        T: DeserializeKey<K>,
        U: Extend<T>,
    {
        while !self.is_empty() {
            let value = self.deserialize()?;
            set.extend(iter::once(value));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
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

impl<K> fmt::Debug for SetDeserializer<'_, '_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("SetDeserializer");

        f.field("buf", &self.buf);
        f.field("len", &self.len);

        f.finish()
    }
}
