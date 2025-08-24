use super::Serializer;
use crate::buf_ext::BufMutExt;
use crate::tags::{KeyTag, KeyTagImpl, Tag};
use crate::{Serialize, SerializeError, SerializeKey, ValueKind};
use bytes::BytesMut;
use std::fmt;
use std::marker::PhantomData;

pub struct Map1Serializer<'a, K> {
    buf: &'a mut BytesMut,
    num_elems: u32,
    depth: u8,
    _key: PhantomData<K>,
}

impl<'a, K: KeyTag> Map1Serializer<'a, K> {
    pub(super) fn new(
        buf: &'a mut BytesMut,
        num_elems: usize,
        depth: u8,
    ) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            buf.put_discriminant_u8(K::Impl::VALUE_KIND_MAP1);
            buf.put_varint_u32_le(num_elems as u32);

            Ok(Self {
                buf,
                num_elems: num_elems as u32,
                depth,
                _key: PhantomData,
            })
        } else {
            Err(SerializeError::Overflow)
        }
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems as usize
    }

    pub fn requires_additional_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn serialize<T: Tag>(
        &mut self,
        key: &(impl SerializeKey<K> + ?Sized),
        value: impl Serialize<T>,
    ) -> Result<&mut Self, SerializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;

            K::Impl::serialize_key(key.try_as_key()?, self.buf)?;

            let serializer = Serializer::new(self.buf, self.depth)?;
            serializer.serialize(value)?;

            Ok(self)
        } else {
            Err(SerializeError::TooManyElements)
        }
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        if self.num_elems == 0 {
            Ok(())
        } else {
            Err(SerializeError::TooFewElements)
        }
    }
}

impl<K> fmt::Debug for Map1Serializer<'_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Map1Serializer");

        f.field("buf", &self.buf);
        f.field("num_elems", &self.num_elems);
        f.field("depth", &self.depth);

        f.finish()
    }
}

pub struct Map2Serializer<'a, K> {
    buf: &'a mut BytesMut,
    depth: u8,
    _key: PhantomData<K>,
}

impl<'a, K: KeyTag> Map2Serializer<'a, K> {
    pub(super) fn new(buf: &'a mut BytesMut, depth: u8) -> Result<Self, SerializeError> {
        buf.put_discriminant_u8(K::Impl::VALUE_KIND_MAP2);

        Ok(Self {
            buf,
            depth,
            _key: PhantomData,
        })
    }

    pub fn serialize<T: Tag>(
        &mut self,
        key: &(impl SerializeKey<K> + ?Sized),
        value: impl Serialize<T>,
    ) -> Result<&mut Self, SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::Some);

        K::Impl::serialize_key(key.try_as_key()?, self.buf)?;

        let serializer = Serializer::new(self.buf, self.depth)?;
        serializer.serialize(value)?;

        Ok(self)
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::None);
        Ok(())
    }
}

impl<K> fmt::Debug for Map2Serializer<'_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Map2Serializer");

        f.field("buf", &self.buf);
        f.field("depth", &self.depth);

        f.finish()
    }
}
