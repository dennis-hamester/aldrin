#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_debug_implementations)]

mod broker;
mod conn;
mod conn_id;
mod serial_map;
#[cfg(test)]
mod test;

pub use broker::{Broker, BrokerError, BrokerHandle};
pub use conn::{Connection, ConnectionError, ConnectionHandle, EstablishError};
