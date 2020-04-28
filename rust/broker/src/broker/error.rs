use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum BrokerError {
    BrokerShutdown,
    FifoOverflow,
}

impl fmt::Display for BrokerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BrokerError::BrokerShutdown => f.write_str("broker shutdown"),
            BrokerError::FifoOverflow => f.write_str("fifo overflow"),
        }
    }
}

impl Error for BrokerError {}
