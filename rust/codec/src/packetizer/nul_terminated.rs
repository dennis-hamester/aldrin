#[cfg(test)]
mod test;

use super::Packetizer;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub struct NulTerminated {
    max_len: usize,
    cur: usize,
}

impl NulTerminated {
    pub fn new() -> Self {
        NulTerminated {
            max_len: u32::MAX as usize,
            cur: 0,
        }
    }

    pub fn with_max_length(max_len: usize) -> Self {
        assert!(max_len > 1);
        NulTerminated { max_len, cur: 0 }
    }

    pub fn max_length(&self) -> usize {
        self.max_len
    }
}

impl Default for NulTerminated {
    fn default() -> Self {
        NulTerminated::new()
    }
}

impl Packetizer for NulTerminated {
    type Error = NulTerminatedError;

    fn encode(&mut self, data: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if data.len() > (self.max_len - 1) {
            return Err(NulTerminatedError);
        }
        debug_assert!(!data.contains(&0));
        dst.reserve(data.len() + 1);
        dst.extend_from_slice(&data);
        dst.put_u8(0);
        Ok(())
    }

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<BytesMut>, Self::Error> {
        debug_assert!(src.len() >= self.cur);
        let nul_idx = src[self.cur..src.len()]
            .iter()
            .position(|&x| x == 0)
            .map(|nul_idx| self.cur + nul_idx);

        match nul_idx {
            Some(nul_idx) if nul_idx > (self.max_len - 1) => Err(NulTerminatedError),

            Some(nul_idx) => {
                let packet = src.split_to(nul_idx);
                src.advance(1);
                self.cur = 0;
                Ok(Some(packet))
            }

            None if src.len() > (self.max_len - 1) => Err(NulTerminatedError),

            None => {
                self.cur = src.len();
                Ok(None)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NulTerminatedError;

impl fmt::Display for NulTerminatedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid packet size")
    }
}

impl StdError for NulTerminatedError {}
