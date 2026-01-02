use crate::buf_ext::BufMutExt;
use crate::tags::{KeyTag, KeyTagImpl};
use crate::{SerializeError, SerializeKey, ValueKind};
use bytes::BytesMut;
use std::fmt;
use std::marker::PhantomData;

pub struct Set1Serializer<'a, K> {
    buf: &'a mut BytesMut,
    num_elems: u32,
    _key: PhantomData<K>,
}

impl<'a, K: KeyTag> Set1Serializer<'a, K> {
    pub(super) fn new(buf: &'a mut BytesMut, num_elems: usize) -> Result<Self, SerializeError> {
        let num_elems = u32::try_from(num_elems).map_err(|_| SerializeError::Overflow)?;

        buf.put_discriminant_u8(K::Impl::VALUE_KIND_SET1);
        buf.put_varint_u32_le(num_elems);

        Ok(Self {
            buf,
            num_elems,
            _key: PhantomData,
        })
    }

    pub fn remaining_elements(&self) -> usize {
        self.num_elems as usize
    }

    pub fn requires_additional_elements(&self) -> bool {
        self.num_elems > 0
    }

    pub fn serialize(
        &mut self,
        value: &(impl SerializeKey<K> + ?Sized),
    ) -> Result<&mut Self, SerializeError> {
        if self.num_elems > 0 {
            self.num_elems -= 1;
            K::Impl::serialize_key(value.try_as_key()?, self.buf)?;
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

impl<T> fmt::Debug for Set1Serializer<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Set1Serializer");

        f.field("buf", &self.buf);
        f.field("num_elems", &self.num_elems);

        f.finish()
    }
}

pub struct Set2Serializer<'a, K> {
    buf: &'a mut BytesMut,
    _key: PhantomData<K>,
}

impl<'a, K: KeyTag> Set2Serializer<'a, K> {
    pub(super) fn new(buf: &'a mut BytesMut) -> Self {
        buf.put_discriminant_u8(K::Impl::VALUE_KIND_SET2);

        Self {
            buf,
            _key: PhantomData,
        }
    }

    pub fn serialize(
        &mut self,
        value: &(impl SerializeKey<K> + ?Sized),
    ) -> Result<&mut Self, SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::Some);
        K::Impl::serialize_key(value.try_as_key()?, self.buf)?;
        Ok(self)
    }

    pub fn finish(self) -> Result<(), SerializeError> {
        self.buf.put_discriminant_u8(ValueKind::None);
        Ok(())
    }
}

impl<T> fmt::Debug for Set2Serializer<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Set2Serializer");

        f.field("buf", &self.buf);

        f.finish()
    }
}
