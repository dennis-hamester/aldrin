use crate::{low_level, Error};
use aldrin_core::tags::PrimaryTag;
use aldrin_core::{Deserialize, SerializedValue, SerializedValueSlice};
use std::fmt;
use std::marker::PhantomData;
use std::time::Instant;

/// Reply of a call.
pub struct Reply<T, E> {
    inner: low_level::Reply,
    phantom: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Reply<T, E> {
    pub(crate) fn new(inner: low_level::Reply) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Casts the reply to a different result type.
    pub fn cast<T2, E2>(self) -> Reply<T2, E2> {
        Reply::new(self.inner)
    }

    /// Extracts the inner low-level [`Reply`](low_level::Reply).
    pub fn into_low_level(self) -> low_level::Reply {
        self.inner
    }

    /// Returns the reply's function id.
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    /// Returns the timestamp when the reply was received.
    pub fn timestamp(&self) -> Instant {
        self.inner.timestamp()
    }

    /// Returns the reply's arguments as slices.
    pub fn args(&self) -> Result<&SerializedValueSlice, &SerializedValueSlice> {
        self.inner.args()
    }

    /// Takes out the reply's arguments and leaves an
    /// [empty `SerializedValue`](SerializedValue::empty) in its place.
    pub fn take_args(&mut self) -> Result<SerializedValue, SerializedValue> {
        self.inner.take_args()
    }
}

impl<T, E> Reply<T, E>
where
    T: PrimaryTag + Deserialize<T::Tag>,
    E: PrimaryTag + Deserialize<E::Tag>,
{
    /// Deserializes the reply's arguments.
    pub fn deserialize(&self) -> Result<Result<T, E>, Error> {
        self.deserialize_as()
    }
}

impl<T: PrimaryTag, E: PrimaryTag> Reply<T, E> {
    /// Deserializes the reply's arguments.
    pub fn deserialize_as<T2, E2>(&self) -> Result<Result<T2, E2>, Error>
    where
        T2: Deserialize<T::Tag>,
        E2: Deserialize<E::Tag>,
    {
        self.inner.deserialize_as().map_err(Error::invalid_reply)
    }
}

impl<T, E> Clone for Reply<T, E> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

impl<T, E> fmt::Debug for Reply<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Reply").field("inner", &self.inner).finish()
    }
}
