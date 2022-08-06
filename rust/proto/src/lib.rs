#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]

mod ids;
mod message;
mod value;

pub mod transport;

pub use ids::{ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid};
pub use message::*;
pub use transport::{AsyncTransport, AsyncTransportExt};
pub use value::{Bytes, ConversionError, FromValue, IntoValue, Value};

pub const VERSION: u32 = 7;
