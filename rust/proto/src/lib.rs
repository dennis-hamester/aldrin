#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_debug_implementations)]

mod key_value;
mod message;
mod transport;
mod value;

use std::error::Error;
use std::fmt;

pub use key_value::{FromKeyValue, IntoKeyValue, KeyValue};
pub use message::*;
pub use transport::AsyncTransport;
pub use value::{FromValue, IntoValue, Value};

pub const VERSION: u32 = 1;

#[derive(Debug, Clone, Copy)]
pub struct ConversionError;

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid conversion from Value or KeyValue")
    }
}

impl Error for ConversionError {}
