use std::error::Error;
use std::fmt;

/// Broker has shut down.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BrokerShutdown;

impl fmt::Display for BrokerShutdown {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("broker shutdown")
    }
}

impl Error for BrokerShutdown {}
