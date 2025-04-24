use aldrin_core::tags::{PrimaryTag, Tag};
use aldrin_core::{Deserialize, DeserializeError, SerializedValue, SerializedValueSlice, Value};
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

    /// Takes out the reply's arguments and leaves an
    /// [empty `SerializedValue`](SerializedValue::empty) in its place.
    pub fn take_args(&mut self) -> Result<SerializedValue, SerializedValue> {
        match self.args {
            Ok(ref mut args) => Ok(args.take()),
            Err(ref mut args) => Err(args.take()),
        }
    }

    /// Returns the arguments of the reply.
    pub fn into_args(self) -> Result<SerializedValue, SerializedValue> {
        self.args
    }

    /// Casts the reply to a high-level [`Reply`](crate::Reply).
    pub fn cast<T, E>(self) -> crate::Reply<T, E> {
        crate::Reply::new(self)
    }

    /// Deserializes the arguments of the reply.
    pub fn deserialize_as<T, U, E, F>(&self) -> Result<Result<U, F>, DeserializeError>
    where
        T: Tag,
        U: Deserialize<T>,
        E: Tag,
        F: Deserialize<E>,
    {
        match self.args {
            Ok(ref ok) => ok.deserialize_as().map(Ok),
            Err(ref err) => err.deserialize_as().map(Err),
        }
    }

    /// Deserializes the arguments of the reply.
    pub fn deserialize<T, E>(&self) -> Result<Result<T, E>, DeserializeError>
    where
        T: PrimaryTag + Deserialize<T::Tag>,
        E: PrimaryTag + Deserialize<E::Tag>,
    {
        self.deserialize_as()
    }

    /// Deserializes the arguments of the reply as generic [`Value`s](Value).
    pub fn deserialize_as_value(&self) -> Result<Result<Value, Value>, DeserializeError> {
        self.deserialize()
    }
}
