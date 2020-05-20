#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_debug_implementations)]

mod message;
mod value;

pub mod transport;

pub use message::*;
pub use transport::AsyncTransport;
pub use value::{Bytes, ConversionError, FromValue, IntoValue, Value};

pub const VERSION: u32 = 3;
