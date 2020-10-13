#![deny(broken_intra_doc_links)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "tokio-codec")]
mod tokio_codec;

pub mod filter;
pub mod packetizer;
pub mod serializer;

pub use filter::{Filter, FilterExt};
pub use packetizer::{Packetizer, PacketizerExt};
pub use serializer::{Serializer, SerializerExt};
#[cfg(feature = "tokio-codec")]
pub use tokio_codec::{TokioCodec, TokioCodecError};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Endian {
    Big,
    Little,
}
