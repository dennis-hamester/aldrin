use super::Serializer;
use crate::buf_ext::BufMutExt;
use crate::tags::{KeyTag, KeyTagImpl, Tag};
use crate::{Serialize, SerializeError, SerializeKey};
use bytes::BytesMut;
use std::fmt;
use std::marker::PhantomData;

pub struct MapSerializer<'a, K> {
    buf: &'a mut BytesMut,
    num_elems: u32,
    depth: u8,
    _key: PhantomData<K>,
}

impl<'a, K: KeyTag> MapSerializer<'a, K> {
    pub(super) fn new(
        mut buf: &'a mut BytesMut,
        num_elems: usize,
        depth: u8,
    ) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            K::Impl::serialize_map_value_kind(&mut buf);
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

    pub fn serialize<L: SerializeKey<K> + ?Sized, T: Tag, U: Serialize<T>>(
        &mut self,
        key: &L,
        value: U,
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

impl<K> fmt::Debug for MapSerializer<'_, K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("MapSerializer");

        f.field("buf", &self.buf);
        f.field("num_elems", &self.num_elems);
        f.field("depth", &self.depth);

        f.finish()
    }
}
