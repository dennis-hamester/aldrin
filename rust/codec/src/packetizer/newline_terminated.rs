#[cfg(test)]
mod test;

use super::Packetizer;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub struct NewlineTerminated {
    max_len: usize,
    cur: usize,
}

impl NewlineTerminated {
    pub fn new() -> Self {
        NewlineTerminated {
            max_len: u32::MAX as usize,
            cur: 0,
        }
    }

    pub fn with_max_length(max_len: usize) -> Self {
        assert!(max_len > 1);
        NewlineTerminated { max_len, cur: 0 }
    }

    pub fn max_length(&self) -> usize {
        self.max_len
    }
}

impl Default for NewlineTerminated {
    fn default() -> Self {
        NewlineTerminated::new()
    }
}

impl Packetizer for NewlineTerminated {
    type Error = NewlineTerminatedError;

    fn encode(&mut self, data: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if data.len() > (self.max_len - 1) {
            return Err(NewlineTerminatedError);
        }
        debug_assert!(!data.contains(&b'\n'));
        dst.reserve(data.len() + 1);
        dst.extend_from_slice(&data);
        dst.put_u8(b'\n');
        Ok(())
    }

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<BytesMut>, Self::Error> {
        debug_assert!(src.len() >= self.cur);
        let lf_idx = src[self.cur..src.len()]
            .iter()
            .position(|&x| x == b'\n')
            .map(|lf_idx| self.cur + lf_idx);

        match lf_idx {
            Some(lf_idx) if lf_idx > (self.max_len - 1) => Err(NewlineTerminatedError),

            Some(lf_idx) => {
                let packet = if (lf_idx > 0) && (src[lf_idx - 1] == b'\r') {
                    let packet = src.split_to(lf_idx - 1);
                    src.advance(2);
                    packet
                } else {
                    let packet = src.split_to(lf_idx);
                    src.advance(1);
                    packet
                };
                self.cur = 0;
                Ok(Some(packet))
            }

            None if src.len() > (self.max_len - 1) => Err(NewlineTerminatedError),

            None => {
                self.cur = src.len();
                Ok(None)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NewlineTerminatedError;

impl fmt::Display for NewlineTerminatedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid packet size")
    }
}

impl StdError for NewlineTerminatedError {}
