#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "bincode-serializer")]
mod bincode;
#[cfg(feature = "json-serializer")]
mod json;
#[cfg(feature = "tokio-codec")]
mod tokio_codec;

pub mod filter;
pub mod packetizer;
pub mod serializer;

#[cfg(feature = "bincode-serializer")]
pub use self::bincode::{BincodeError, BincodeSerializer};
pub use filter::{Filter, FilterExt};
#[cfg(feature = "json-serializer")]
pub use json::{JsonError, JsonSerializer};
pub use packetizer::{Packetizer, PacketizerExt};
pub use serializer::{Serializer, SerializerExt};
#[cfg(feature = "tokio-codec")]
pub use tokio_codec::{TokioCodec, TokioCodecError};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Endian {
    Big,
    Little,
}
