#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_debug_implementations)]

mod message;
mod value;

pub mod transport;

pub use message::*;
pub use transport::{AsyncTransport, AsyncTransportExt};
pub use value::{Bytes, ConversionError, FromValue, IntoValue, ObjectId, Value};

pub const VERSION: u32 = 5;
