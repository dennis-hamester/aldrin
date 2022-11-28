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
    use super::super::{
        ChannelEnd, CreateChannel, Message, MessageOps, Shutdown, UnsubscribeServices,
    };
    use super::Packetizer;
    use bytes::Buf;
    use std::mem::MaybeUninit;

    #[test]
    fn extend_from_slice() {
        let msg1 = Message::Shutdown(Shutdown);
        let msg2 = Message::UnsubscribeServices(UnsubscribeServices);
        let msg3 = Message::CreateChannel(CreateChannel {
            serial: 0,
            claim: ChannelEnd::Sender,
        });

        let mut serialized = msg1.clone().serialize_message().unwrap();
        let tmp = msg2.clone().serialize_message().unwrap();
        serialized.extend_from_slice(&tmp);
        let tmp = msg3.clone().serialize_message().unwrap();
        serialized.extend_from_slice(&tmp);
        assert_eq!(
            serialized[..],
            [5, 0, 0, 0, 2, 5, 0, 0, 0, 18, 7, 0, 0, 0, 31, 0, 0]
        );

        let mut packetizer = Packetizer::new();
        assert_eq!(packetizer.next_message(), None);

        packetizer.extend_from_slice(&serialized[..3]);
        serialized.advance(3);
        assert_eq!(packetizer.next_message(), None);

        packetizer.extend_from_slice(&serialized[..8]);
        serialized.advance(8);
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
    fn space_capacity_mut() {
        fn write_slice(dst: &mut [MaybeUninit<u8>], src: &[u8]) {
            for (&src, dst) in src.iter().zip(dst) {
                dst.write(src);
            }
        }

        let msg1 = Message::Shutdown(Shutdown);
        let msg2 = Message::UnsubscribeServices(UnsubscribeServices);
        let msg3 = Message::CreateChannel(CreateChannel {
            serial: 0,
            claim: ChannelEnd::Sender,
        });

        let mut serialized = msg1.clone().serialize_message().unwrap();
        let tmp = msg2.clone().serialize_message().unwrap();
        serialized.extend_from_slice(&tmp);
        let tmp = msg3.clone().serialize_message().unwrap();
        serialized.extend_from_slice(&tmp);
        assert_eq!(
            serialized[..],
            [5, 0, 0, 0, 2, 5, 0, 0, 0, 18, 7, 0, 0, 0, 31, 0, 0]
        );

        let mut packetizer = Packetizer::new();
        assert_eq!(packetizer.next_message(), None);

        write_slice(packetizer.spare_capacity_mut(), &serialized[..3]);
        unsafe {
            packetizer.bytes_written(3);
        }
        serialized.advance(3);
        assert_eq!(packetizer.next_message(), None);

        write_slice(packetizer.spare_capacity_mut(), &serialized[..8]);
        unsafe {
            packetizer.bytes_written(8);
        }
        serialized.advance(8);
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
