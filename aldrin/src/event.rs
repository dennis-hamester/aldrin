use crate::{low_level, Error, UnknownEvent};
use aldrin_core::tags::PrimaryTag;
use aldrin_core::{Deserialize, SerializedValue, SerializedValueSlice};
use std::fmt;
use std::marker::PhantomData;
use std::time::Instant;

/// Event emitted by a service.
pub struct Event<T> {
    inner: low_level::Event,
    phantom: PhantomData<fn() -> T>,
}

impl<T> Event<T> {
    pub(crate) fn new(inner: low_level::Event) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    /// Returns the event's id.
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    /// Returns the timestamp when the event was received.
    pub fn timestamp(&self) -> Instant {
        self.inner.timestamp()
    }

    /// Casts the event to a different type.
    pub fn cast<T2>(self) -> Event<T2> {
        Event::new(self.inner)
    }

    /// Extracts the inner low-level [`Event`](low_level::Event).
    pub fn into_low_level(self) -> low_level::Event {
        self.inner
    }

    /// Converts this event into an [`UnknownEvent`].
    pub fn into_unknown_call(self) -> UnknownEvent {
        self.inner.into_unknown_event()
    }

    /// Returns a slice to the event's serialized arguments.
    pub fn args(&self) -> &SerializedValueSlice {
        self.inner.args()
    }

    /// Takes out the event's arguments and leaves an
    /// [empty `SerializedValue`](SerializedValue::empty) in its place.
    pub fn take_args(&mut self) -> SerializedValue {
        self.inner.take_args()
    }
}

impl<T: PrimaryTag + Deserialize<T::Tag>> Event<T> {
    /// Deserializes the event's arguments.
    pub fn deserialize(&self) -> Result<T, Error> {
        self.deserialize_as()
    }
}

impl<T: PrimaryTag> Event<T> {
    /// Deserializes the event's arguments.
    pub fn deserialize_as<T2: Deserialize<T::Tag>>(&self) -> Result<T2, Error> {
        self.inner
            .deserialize_as()
            .map_err(|e| Error::invalid_arguments(self.id(), Some(e)))
    }
}

impl<T> Clone for Event<T> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

impl<T> fmt::Debug for Event<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Event").field("inner", &self.inner).finish()
    }
}
