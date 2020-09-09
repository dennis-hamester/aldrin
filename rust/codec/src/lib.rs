#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "bincode-serializer")]
mod bincode;
#[cfg(feature = "json")]
mod json;
mod length_prefixed;
mod noop_filter;
#[cfg(feature = "tokio-io")]
mod tokio_io;

pub mod filter;
pub mod packetizer;
pub mod serializer;

#[cfg(feature = "bincode-serializer")]
pub use self::bincode::{BincodeError, BincodeSerializer};
pub use filter::{Filter, FilterExt};
#[cfg(feature = "json")]
pub use json::{JsonError, JsonSerializer};
pub use length_prefixed::{LengthPrefixed, LengthPrefixedBuilder, LengthPrefixedError};
pub use noop_filter::NoopFilter;
pub use packetizer::{Packetizer, PacketizerExt};
pub use serializer::{Serializer, SerializerExt};
#[cfg(feature = "tokio-io")]
pub use tokio_io::{TokioCodec, TokioCodecError};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Endian {
    Big,
    Little,
}
