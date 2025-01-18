use crate::core::{Deserialize, DeserializeError, SerializedValue, SerializedValueSlice, Value};
use crate::reply::Reply as HlReply;
use std::time::Instant;

/// Reply of a call.
#[derive(Debug, Clone)]
pub struct Reply {
    id: u32,
    timestamp: Instant,
    args: Result<SerializedValue, SerializedValue>,
}

impl Reply {
    pub(crate) fn new(
        id: u32,
        timestamp: Instant,
        args: Result<SerializedValue, SerializedValue>,
    ) -> Self {
        Self {
            id,
            timestamp,
            args,
        }
    }

    /// Returns the reply's function id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the timestamp when the reply was received.
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    /// Returns the reply's arguments as slices.
    pub fn args(&self) -> Result<&SerializedValueSlice, &SerializedValueSlice> {
        match self.args {
            Ok(ref ok) => Ok(ok),
            Err(ref err) => Err(err),
        }
    }

    /// Returns the arguments of the reply.
    pub fn into_args(self) -> Result<SerializedValue, SerializedValue> {
        self.args
    }

    /// Deserializes the arguments of the reply.
    pub fn deserialize<T: Deserialize, E: Deserialize>(
        &self,
    ) -> Result<Result<T, E>, DeserializeError> {
        match self.args {
            Ok(ref ok) => ok.deserialize().map(Ok),
            Err(ref err) => err.deserialize().map(Err),
        }
    }

    /// Deserializes the arguments of the reply as generic [`Value`s](Value).
    pub fn deserialize_as_value(&self) -> Result<Result<Value, Value>, DeserializeError> {
        self.deserialize()
    }

    /// Deserializes the arguments and casts the reply to a high-level [`Reply`](HlReply).
    pub fn deserialize_and_cast<T, E>(&self) -> Result<crate::reply::Reply<T, E>, DeserializeError>
    where
        T: Deserialize,
        E: Deserialize,
    {
        self.deserialize()
            .map(|args| HlReply::new(self.id, self.timestamp, args))
    }
}
