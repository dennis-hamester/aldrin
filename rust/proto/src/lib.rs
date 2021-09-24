#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]

mod message;
mod value;

pub mod transport;

pub use message::*;
pub use transport::{AsyncTransport, AsyncTransportExt};
pub use value::{Bytes, ConversionError, FromValue, IntoValue, ObjectId, ServiceId, Value};

pub const VERSION: u32 = 6;
