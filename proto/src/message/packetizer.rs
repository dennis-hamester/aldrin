use bytes::{Buf, BytesMut};
use std::mem::MaybeUninit;

const MIN_RESERVE_CAPACITY: usize = 64 * 1024;
const MAX_RESERVE_CAPACITY: usize = 4 * 1024 * 1024;

/// Splits a continuous stream of bytes into individual messages.
#[derive(Debug)]
pub struct Packetizer {
    buf: BytesMut,
    len: Option<usize>,
}

impl Packetizer {
    pub fn new() -> Self {
        Self {
            buf: BytesMut::new(),
            len: None,
        }
    }

    pub fn extend_from_slice(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes.as_ref());
    }

    /// Returns a slice of uninitialized bytes at the end of the internal buffer.
    ///
    /// This function, together with [`bytes_written`](Self::bytes_written), make it possible to
    /// fill the packetizer without an intermediate buffer.
    ///
    /// The slice returned by this function is guaranteed to be non-empty.
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        if let Some(len) = self.len {
            if self.buf.capacity() < len {
                let reserve =
                    (len - self.buf.len()).clamp(MIN_RESERVE_CAPACITY, MAX_RESERVE_CAPACITY);
                self.buf.reserve(reserve);
            }
        } else if self.buf.capacity() == self.buf.len() {
            self.buf.reserve(MIN_RESERVE_CAPACITY);
        }

        let slice = self.buf.spare_capacity_mut();
        debug_assert!(!slice.is_empty());
        slice
    }

    /// Asserts that the next `len` bytes have been initialized.
    ///
    /// # Safety
    ///
    /// You must ensure that prior to calling this function, at least `len` bytes of the slice
    /// returned by [`spare_capacity_mut`](Self::spare_capacity_mut) have been initialized.
    pub unsafe fn bytes_written(&mut self, len: usize) {
        unsafe {
            self.buf.set_len(self.buf.len() + len);
        }
    }

    pub fn next_message(&mut self) -> Option<BytesMut> {
        if self.buf.len() < 4 {
            return None;
        }

        let len = match self.len {
            Some(len) => len,

            None => {
                let len = (&self.buf[..4]).get_u32_le() as usize;
                self.len = Some(len);
                len
            }
        };

        if self.buf.len() >= len {
            let msg = self.buf.split_to(len);
            self.len = None;
            Some(msg)
        } else {
            None
        }
    }
}

impl Default for Packetizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::super::{CreateChannel, CreateObject, Message, MessageOps, Shutdown};
    use super::Packetizer;
    use crate::channel_end::ChannelEndWithCapacity;
    use crate::ids::ObjectUuid;
    use bytes::Buf;
    use std::mem::MaybeUninit;
    use uuid::uuid;

    #[test]
    fn extend_from_slice() {
        let msg1 = Message::Shutdown(Shutdown);
        let msg2 = Message::CreateObject(CreateObject {
            serial: 1,
            uuid: ObjectUuid(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
        });
        let msg3 = Message::CreateChannel(CreateChannel {
            serial: 0,
            end: ChannelEndWithCapacity::Sender,
        });

        let mut serialized = msg1.clone().serialize_message().unwrap();
        let tmp = msg2.clone().serialize_message().unwrap();
        serialized.extend_from_slice(&tmp);
        let tmp = msg3.clone().serialize_message().unwrap();
        serialized.extend_from_slice(&tmp);
        assert_eq!(
            serialized[..],
            [
                5, 0, 0, 0, 2, 22, 0, 0, 0, 3, 1, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e,
                0xb4, 0xbf, 0x37, 0x38, 0x76, 0x52, 0x3d, 0x1b, 7, 0, 0, 0, 19, 0, 0,
            ]
        );

        let mut packetizer = Packetizer::new();
        assert_eq!(packetizer.next_message(), None);

        packetizer.extend_from_slice(&serialized[..3]);
        serialized.advance(3);
        assert_eq!(packetizer.next_message(), None);

        packetizer.extend_from_slice(&serialized[..25]);
        serialized.advance(25);
        let msg1_serialized = packetizer.next_message().unwrap();
        assert_eq!(Message::deserialize_message(msg1_serialized), Ok(msg1));
        let msg2_serialized = packetizer.next_message().unwrap();
        assert_eq!(Message::deserialize_message(msg2_serialized), Ok(msg2));
        assert_eq!(packetizer.next_message(), None);

        packetizer.extend_from_slice(&serialized[..6]);
        serialized.advance(6);
        let msg3_serialized = packetizer.next_message().unwrap();
        assert_eq!(Message::deserialize_message(msg3_serialized), Ok(msg3));
        assert_eq!(packetizer.next_message(), None);

        assert_eq!(serialized[..], []);
    }

    #[test]
    fn spare_capacity_mut() {
        fn write_slice(dst: &mut [MaybeUninit<u8>], src: &[u8]) {
            for (&src, dst) in src.iter().zip(dst) {
                dst.write(src);
            }
        }

        let msg1 = Message::Shutdown(Shutdown);
        let msg2 = Message::CreateObject(CreateObject {
            serial: 1,
            uuid: ObjectUuid(uuid!("b7c3be13-5377-466e-b4bf-373876523d1b")),
        });
        let msg3 = Message::CreateChannel(CreateChannel {
            serial: 0,
            end: ChannelEndWithCapacity::Sender,
        });

        let mut serialized = msg1.clone().serialize_message().unwrap();
        let tmp = msg2.clone().serialize_message().unwrap();
        serialized.extend_from_slice(&tmp);
        let tmp = msg3.clone().serialize_message().unwrap();
        serialized.extend_from_slice(&tmp);
        assert_eq!(
            serialized[..],
            [
                5, 0, 0, 0, 2, 22, 0, 0, 0, 3, 1, 0xb7, 0xc3, 0xbe, 0x13, 0x53, 0x77, 0x46, 0x6e,
                0xb4, 0xbf, 0x37, 0x38, 0x76, 0x52, 0x3d, 0x1b, 7, 0, 0, 0, 19, 0, 0,
            ]
        );

        let mut packetizer = Packetizer::new();
        assert_eq!(packetizer.next_message(), None);

        write_slice(packetizer.spare_capacity_mut(), &serialized[..3]);
        unsafe {
            packetizer.bytes_written(3);
        }
        serialized.advance(3);
        assert_eq!(packetizer.next_message(), None);

        write_slice(packetizer.spare_capacity_mut(), &serialized[..25]);
        unsafe {
            packetizer.bytes_written(25);
        }
        serialized.advance(25);
        let msg1_serialized = packetizer.next_message().unwrap();
        assert_eq!(Message::deserialize_message(msg1_serialized), Ok(msg1));
        let msg2_serialized = packetizer.next_message().unwrap();
        assert_eq!(Message::deserialize_message(msg2_serialized), Ok(msg2));
        assert_eq!(packetizer.next_message(), None);

        write_slice(packetizer.spare_capacity_mut(), &serialized[..6]);
        unsafe {
            packetizer.bytes_written(6);
        }
        serialized.advance(6);
        let msg3_serialized = packetizer.next_message().unwrap();
        assert_eq!(Message::deserialize_message(msg3_serialized), Ok(msg3));
        assert_eq!(packetizer.next_message(), None);

        assert_eq!(serialized[..], []);
    }
}
