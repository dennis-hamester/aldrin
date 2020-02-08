use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum BrokerError {
    InternalError,
    BrokerShutdown,
}

impl fmt::Display for BrokerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BrokerError::InternalError => f.write_str("internal error"),
            BrokerError::BrokerShutdown => f.write_str("broker shutdown"),
        }
    }
}

impl Error for BrokerError {}
