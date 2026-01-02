use super::Deserializer;
use crate::buf_ext::ValueBufExt;
use crate::tags::{KeyTag, KeyTagImpl, Tag};
use crate::{Deserialize, DeserializeError, DeserializeKey, ValueKind};
use std::marker::PhantomData;
use std::{fmt, iter};

#[derive(Debug)]
pub enum MapDeserializer<'a, 'b, K> {
    V1(Map1Deserializer<'a, 'b, K>),
    V2(Map2Deserializer<'a, 'b, K>),
}

impl<'a, 'b, K: KeyTag> MapDeserializer<'a, 'b, K> {
    pub(super) fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        let kind = buf.try_get_discriminant_u8::<ValueKind>()?;

        if kind == K::Impl::VALUE_KIND_MAP1 {
            Map1Deserializer::new_without_value_kind(buf, depth).map(Self::V1)
        } else if kind == K::Impl::VALUE_KIND_MAP2 {
            Ok(Self::V2(Map2Deserializer::new_without_value_kind(
                buf, depth,
            )))
        } else {
            Err(DeserializeError::UnexpectedValue)
        }
    }

    pub fn deserialize<L: DeserializeKey<K>>(
        &mut self,
    ) -> Result<Option<MapElementDeserializer<'_, 'b, L>>, DeserializeError> {
        match self {
            Self::V1(deserializer) => deserializer.deserialize(),
            Self::V2(deserializer) => deserializer.deserialize(),
        }
    }

    pub fn deserialize_element<L, T, U>(&mut self) -> Result<Option<(L, U)>, DeserializeError>
    where
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
    {
        match self {
            Self::V1(deserializer) => deserializer.deserialize_element(),
            Self::V2(deserializer) => deserializer.deserialize_element(),
        }
    }

    pub fn deserialize_extend<L, T, U, V>(self, map: &mut V) -> Result<(), DeserializeError>
    where
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
        V: Extend<(L, U)>,
    {
        match self {
            Self::V1(deserializer) => deserializer.deserialize_extend(map),
            Self::V2(deserializer) => deserializer.deserialize_extend(map),
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

pub struct Map1Deserializer<'a, 'b, K> {
    buf: &'a mut &'b [u8],
    len: u32,
    depth: u8,
    _key: PhantomData<K>,
}

impl<'a, 'b, K: KeyTag> Map1Deserializer<'a, 'b, K> {
    pub(super) fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(K::Impl::VALUE_KIND_MAP1)?;
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
    ) -> Result<Option<MapElementDeserializer<'_, 'b, L>>, DeserializeError> {
        if self.is_empty() {
            Ok(None)
        } else {
            self.len -= 1;
            MapElementDeserializer::new(self.buf, self.depth).map(Some)
        }
    }

    pub fn deserialize_element<L, T, U>(&mut self) -> Result<Option<(L, U)>, DeserializeError>
    where
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
    {
        match self.deserialize()? {
            Some(elem) => elem.deserialize().map(Some),
            None => Ok(None),
        }
    }

    pub fn deserialize_extend<L, T, U, V>(mut self, map: &mut V) -> Result<(), DeserializeError>
    where
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
        V: Extend<(L, U)>,
    {
        while let Some(kv) = self.deserialize_element()? {
            map.extend(iter::once(kv));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if self.is_empty() {
            Ok(())
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

impl<K> fmt::Debug for Map1Deserializer<'_, '_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("MapDeserializer");

        f.field("buf", &self.buf);
        f.field("len", &self.len);
        f.field("depth", &self.depth);

        f.finish()
    }
}

pub struct Map2Deserializer<'a, 'b, K> {
    buf: &'a mut &'b [u8],
    empty: bool,
    depth: u8,
    _key: PhantomData<K>,
}

impl<'a, 'b, K: KeyTag> Map2Deserializer<'a, 'b, K> {
    pub(super) fn new(buf: &'a mut &'b [u8], depth: u8) -> Result<Self, DeserializeError> {
        buf.ensure_discriminant_u8(K::Impl::VALUE_KIND_MAP2)?;
        Ok(Self::new_without_value_kind(buf, depth))
    }

    pub(super) fn new_without_value_kind(buf: &'a mut &'b [u8], depth: u8) -> Self {
        Self {
            buf,
            empty: false,
            depth,
            _key: PhantomData,
        }
    }

    pub fn deserialize<L: DeserializeKey<K>>(
        &mut self,
    ) -> Result<Option<MapElementDeserializer<'_, 'b, L>>, DeserializeError> {
        if self.empty {
            Ok(None)
        } else {
            #[expect(clippy::wildcard_enum_match_arm)]
            match self.buf.try_get_discriminant_u8()? {
                ValueKind::None => {
                    self.empty = true;
                    Ok(None)
                }

                ValueKind::Some => MapElementDeserializer::new(self.buf, self.depth).map(Some),
                _ => Err(DeserializeError::InvalidSerialization),
            }
        }
    }

    pub fn deserialize_element<L, T, U>(&mut self) -> Result<Option<(L, U)>, DeserializeError>
    where
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
    {
        match self.deserialize()? {
            Some(elem) => elem.deserialize().map(Some),
            None => Ok(None),
        }
    }

    pub fn deserialize_extend<L, T, U, V>(mut self, map: &mut V) -> Result<(), DeserializeError>
    where
        L: DeserializeKey<K>,
        T: Tag,
        U: Deserialize<T>,
        V: Extend<(L, U)>,
    {
        while let Some(kv) = self.deserialize_element()? {
            map.extend(iter::once(kv));
        }

        Ok(())
    }

    pub fn skip_element(&mut self) -> Result<(), DeserializeError> {
        if !self.empty {
            #[expect(clippy::wildcard_enum_match_arm)]
            match self.buf.try_get_discriminant_u8()? {
                ValueKind::None => self.empty = true,

                ValueKind::Some => {
                    K::Impl::skip(self.buf)?;
                    Deserializer::new(self.buf, self.depth)?.skip()?;
                }

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
            #[expect(clippy::wildcard_enum_match_arm)]
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

impl<K> fmt::Debug for Map2Deserializer<'_, '_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("MapDeserializer");

        f.field("buf", &self.buf);
        f.field("empty", &self.empty);
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
