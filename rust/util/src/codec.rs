#[cfg(feature = "bincode-serializer")]
mod bincode;
#[cfg(feature = "json")]
mod json;
mod length_prefixed;
mod serializer;
#[cfg(feature = "tokio-io")]
mod tokio_io;

pub mod packetizer;

#[cfg(feature = "bincode-serializer")]
pub use self::bincode::{BincodeError, BincodeSerializer};
#[cfg(feature = "json")]
pub use json::{JsonError, JsonSerializer};
pub use length_prefixed::{LengthPrefixed, LengthPrefixedBuilder, LengthPrefixedError};
pub use packetizer::{Packetizer, PacketizerExt};
pub use serializer::Serializer;
#[cfg(feature = "tokio-io")]
pub use tokio_io::{TokioCodec, TokioCodecError};

#[derive(Debug, Clone, Copy)]
pub enum Endian {
    Big,
    Little,
}
