use super::Deserializer;
use crate::buf_ext::ValueBufExt;
use crate::tags::{KeyTag, KeyTagImpl, Tag};
use crate::{Deserialize, DeserializeError, DeserializeKey};
use std::marker::PhantomData;
use std::{fmt, iter};

pub struct MapDeserializer<'a, 'b, K> {
    buf: &'a mut &'b [u8],
    len: u32,
    depth: u8,
    _key: PhantomData<K>,
}

impl<'a, 'b, K: KeyTag> MapDeserializer<'a, 'b, K> {
    pub(super) fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        K::Impl::deserialize_map_value_kind(buf)?;
        Self::new_without_value_kind(buf, depth)
    }

    pub(super) fn new_without_value_kind(
        buf: &'a mut &'b [u8],
        depth: u8,
    ) -> Result<Self, DeserializeError> {
        let len = buf.try_get_varint_u32_le()?;

        Ok(Self {
            buf,
            len,
            depth,
            _key: PhantomData,
        })
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn deserialize<L: DeserializeKey<K>>(
        &mut self,
    ) -> Result<MapElementDeserializer<'_, 'b, L>, DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            MapElementDeserializer::new(self.buf, self.depth)
        }
    }

    pub fn deserialize_element<L, T, U>(&mut self) -> Result<(L, U), DeserializeError>
    where
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
    {
        self.deserialize()?.deserialize()
    }

    pub fn deserialize_extend<L, T, U, V>(mut self, map: &mut V) -> Result<(), DeserializeError>
    where
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
        V: Extend<(L, U)>,
    {
        while !self.is_empty() {
            let kv = self.deserialize_element()?;
            map.extend(iter::once(kv));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.is_empty() {
            Err(DeserializeError::NoMoreElements)
        } else {
            self.len -= 1;
            K::Impl::skip(self.buf)?;
            Deserializer::new(self.buf, self.depth)?.skip()
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

impl<K> fmt::Debug for MapDeserializer<'_, '_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("MapDeserializer");

        f.field("buf", &self.buf);
        f.field("len", &self.len);
        f.field("depth", &self.depth);

        f.finish()
    }
}

#[derive(Debug)]
pub struct MapElementDeserializer<'a, 'b, L> {
    buf: &'a mut &'b [u8],
    key: L,
    depth: u8,
}

impl<'a, 'b, L> MapElementDeserializer<'a, 'b, L> {
    fn new<K>(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError>
    where
        K: KeyTag,
        L: DeserializeKey<K>,
    {
        let key = K::Impl::deserialize_key(buf).and_then(L::try_from_key)?;
        Ok(Self { buf, key, depth })
    }

    pub fn key(&self) -> &L {
        &self.key
    }

    pub fn deserialize<T: Tag, U: Deserialize<T>>(self) -> Result<(L, U), DeserializeError> {
        let deserializer = Deserializer::new(self.buf, self.depth)?;
        let value = deserializer.deserialize()?;
        Ok((self.key, value))
    }

    pub fn skip(self) -> Result<(), DeserializeError> {
        Deserializer::new(self.buf, self.depth)?.skip()
    }
}
