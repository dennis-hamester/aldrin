#[cfg(feature = "bincode-serializer")]
mod bincode;
#[cfg(feature = "json")]
mod json;
mod packetizer;
mod serializer;
#[cfg(feature = "tokio-io")]
mod tokio_io;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::error::Error as StdError;
use std::fmt;

#[cfg(feature = "bincode-serializer")]
pub use self::bincode::{BincodeError, BincodeSerializer};
#[cfg(feature = "json")]
pub use json::{JsonError, JsonSerializer};
pub use packetizer::Packetizer;
pub use serializer::Serializer;
#[cfg(feature = "tokio-io")]
pub use tokio_io::{TokioCodec, TokioCodecError};

#[derive(Debug)]
pub struct LengthPrefixed {
    len_size: u8,
    max_len: usize,
    endian: Endian,
    cur: Option<usize>,
}

impl LengthPrefixed {
    pub fn new() -> Self {
        LengthPrefixedBuilder::new().build()
    }

    pub fn builder() -> LengthPrefixedBuilder {
        LengthPrefixedBuilder::new()
    }

    pub fn length_size(&self) -> u8 {
        self.len_size
    }

    pub fn max_length(&self) -> usize {
        self.max_len
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }
}

impl Default for LengthPrefixed {
    fn default() -> Self {
        LengthPrefixed::new()
    }
}

impl Packetizer for LengthPrefixed {
    type Error = LengthPrefixedError;

    fn encode(&mut self, data: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let n = (self.len_size as usize)
            .checked_add(data.len())
            .ok_or(LengthPrefixedError)?;
        if n > self.max_len {
            return Err(LengthPrefixedError);
        }

        dst.reserve(n);
        match self.endian {
            Endian::Big => dst.put_uint(n as u64, self.len_size as usize),
            Endian::Little => dst.put_uint_le(n as u64, self.len_size as usize),
        }
        dst.extend_from_slice(&data);
        Ok(())
    }

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Bytes>, Self::Error> {
        match self.cur {
            Some(len) => {
                if src.len() >= len {
                    self.cur = None;
                    Ok(Some(src.split_to(len).freeze()))
                } else {
                    Ok(None)
                }
            }

            None => {
                if src.len() >= self.len_size as usize {
                    let len = match self.endian {
                        Endian::Big => src.get_uint(self.len_size as usize) as usize,
                        Endian::Little => src.get_uint_le(self.len_size as usize) as usize,
                    };
                    let len = len
                        .checked_sub(self.len_size as usize)
                        .ok_or(LengthPrefixedError)?;

                    if src.len() >= len {
                        Ok(Some(src.split_to(len).freeze()))
                    } else {
                        self.cur = Some(len);
                        src.reserve(len);
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LengthPrefixedBuilder {
    len_size: u8,
    max_len: usize,
    endian: Endian,
}

impl LengthPrefixedBuilder {
    pub fn new() -> Self {
        LengthPrefixedBuilder {
            len_size: 4,
            max_len: u32::max_value() as usize,
            endian: Endian::Big,
        }
    }

    pub fn build(&self) -> LengthPrefixed {
        assert!(self.len_size > 0);
        assert!(self.max_len >= self.len_size as usize);
        let bits_required = 64 - (self.max_len as u64).leading_zeros();
        assert!((self.len_size as u32 * 8) >= bits_required);

        LengthPrefixed {
            len_size: self.len_size,
            max_len: self.max_len,
            endian: self.endian,
            cur: None,
        }
    }

    pub fn length_size(&self) -> u8 {
        self.len_size
    }

    pub fn set_length_size(&mut self, length_size: u8) -> &mut Self {
        self.len_size = length_size;
        self
    }

    pub fn max_length(&self) -> usize {
        self.max_len
    }

    pub fn set_max_length(&mut self, max_length: usize) -> &mut Self {
        self.max_len = max_length;
        self
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }

    pub fn set_endian(&mut self, endian: Endian) -> &mut Self {
        self.endian = endian;
        self
    }

    pub fn big_endian(&mut self) -> &mut Self {
        self.endian = Endian::Big;
        self
    }

    pub fn little_endian(&mut self) -> &mut Self {
        self.endian = Endian::Little;
        self
    }

    pub fn native_endian(&mut self) -> &mut Self {
        if cfg!(target_endian = "big") {
            self.big_endian()
        } else {
            self.little_endian()
        }
    }
}

impl Default for LengthPrefixedBuilder {
    fn default() -> Self {
        LengthPrefixedBuilder::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Debug, Clone, Copy)]
pub struct LengthPrefixedError;

impl fmt::Display for LengthPrefixedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid packet size")
    }
}

impl StdError for LengthPrefixedError {}
