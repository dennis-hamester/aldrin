use crate::buf_ext::BufMutExt;
use crate::tags::{KeyTag, KeyTagImpl};
use crate::{SerializeError, SerializeKey};
use bytes::BytesMut;
use std::fmt;
use std::marker::PhantomData;

pub struct SetSerializer<'a, K> {
    buf: &'a mut BytesMut,
    num_elems: u32,
    _key: PhantomData<K>,
}

impl<'a, K: KeyTag> SetSerializer<'a, K> {
    pub(super) fn new(mut buf: &'a mut BytesMut, num_elems: usize) -> Result<Self, SerializeError> {
        if num_elems <= u32::MAX as usize {
            K::Impl::serialize_set_value_kind(&mut buf);
            buf.put_varint_u32_le(num_elems as u32);

            Ok(Self {
                buf,
                num_elems: num_elems as u32,
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

    pub fn serialize<T: SerializeKey<K> + ?Sized>(
        &mut self,
        value: &T,
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

impl<T> fmt::Debug for SetSerializer<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("SetSerializer");

        f.field("buf", &self.buf);
        f.field("num_elems", &self.num_elems);

        f.finish()
    }
}
