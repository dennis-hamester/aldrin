use crate::event::Event;
use crate::reply::Reply;
use std::time::Instant;

/// Tracks some state of a service.
///
/// `Property` can be used to track the state of some remote variable, which is typically exposed
/// with a getter function and an event that emits changes. Property values are tracked together
/// with a timestamp, which denotes when the value was last changed.
///
/// `Property` also helps to eliminate potential race conditions, particularly at start up, when
/// state is queried from a server.
#[derive(Debug, Clone)]
pub struct Property<T> {
    val: T,
    timestamp: Instant,
}

impl<T: Default> Property<T> {
    /// Creates a new [`Property`] with the default value of `T` and [`Instant::now()`].
    pub fn new() -> Self {
        Self::with_value(T::default())
    }
}

impl<T> Property<T> {
    /// Creates a new [`Property`] with the given value and [`Instant::now()`].
    pub fn with_value(val: T) -> Self {
        Self::with_value_and_timestamp(val, Instant::now())
    }

    /// Creates a new [`Property`] with the given value and timestamp.
    pub fn with_value_and_timestamp(val: T, timestamp: Instant) -> Self {
        Self { val, timestamp }
    }

    /// Creates a new [`Property`] from a [`Reply`].
    ///
    /// An [`Err(_)`](Err) in the [`Reply`] is propagated back.
    pub fn from_reply<E>(reply: Reply<T, E>) -> Result<Self, E> {
        let timestamp = reply.timestamp();
        let val = reply.into_args()?;
        Ok(Self::with_value_and_timestamp(val, timestamp))
    }

    /// Creates a new [`Property`] from an [`Event`].
    pub fn from_event(ev: Event<T>) -> Self {
        let timestamp = ev.timestamp();
        let val = ev.into_args();
        Self::with_value_and_timestamp(val, timestamp)
    }
}

impl<T> Property<Option<T>> {
    /// Creates a new [`Property`] with the value [`Some(val)`](Some) and [`Instant::now()`].
    pub fn with_value_some(val: T) -> Self {
        Self::with_value_and_timestamp_some(val, Instant::now())
    }

    /// Creates a new [`Property`] with the value [`Some(val)`](Some) and the given timestamp.
    pub fn with_value_and_timestamp_some(val: T, timestamp: Instant) -> Self {
        Self {
            val: Some(val),
            timestamp,
        }
    }

    /// Creates a new [`Property`] from a [`Reply`] by wrapping the value in [`Some(_)`](Some).
    pub fn from_reply_some<E>(reply: Reply<T, E>) -> Result<Self, E> {
        let timestamp = reply.timestamp();
        let val = reply.into_args()?;
        Ok(Self::with_value_and_timestamp_some(val, timestamp))
    }

    /// Creates a new [`Property`] from an [`Event`] by wrapping the value in [`Some(_)`](Some).
    pub fn from_event_some(ev: Event<T>) -> Self {
        let timestamp = ev.timestamp();
        let val = ev.into_args();
        Self::with_value_and_timestamp_some(val, timestamp)
    }
}

impl<T> Property<T> {
    /// Returns the current timestamp of the [`Property`].
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    /// Returns the current value of the [`Property`].
    pub fn get(&self) -> &T {
        &self.val
    }

    /// Sets the value to `val` and returns a reference to it.
    ///
    /// The timestamp will be set to [`Instant::now()`].
    pub fn set(&mut self, val: T) -> &T {
        self.timestamp = Instant::now();
        self.val = val;
        &self.val
    }
}

impl<T> Property<Option<T>> {
    /// Sets the value to [`Some(val)`](Some) and returns a reference to the inner value.
    ///
    /// The timestamp will be set to [`Instant::now()`].
    pub fn set_some(&mut self, val: T) -> &T {
        self.timestamp = Instant::now();
        self.val.insert(val)
    }

    /// Sets the value to [`None`].
    ///
    /// The timestamp will be set to [`Instant::now()`].
    pub fn set_none(&mut self) {
        self.timestamp = Instant::now();
        self.val = None;
    }
}

impl<T> Property<T> {
    /// Updates the value to `val` if `timestamp` is newer.
    ///
    /// Returns [`Some(_)`](Some), if the value was updated.
    pub fn update(&mut self, val: T, timestamp: Instant) -> Option<&T> {
        if timestamp > self.timestamp {
            self.timestamp = timestamp;
            self.val = val;
            Some(&self.val)
        } else {
            None
        }
    }

    /// Updates the value from a [`Reply`], if its timestamp is newer.
    ///
    /// Returns:
    /// - [`Err(_)`](Err), if the [`Reply`] contains an error.
    /// - [`Ok(`](Ok)[`Some(_)`](Some)[`)`](Ok), if the [`Reply`] contains an [`Ok(_)`](Ok) value
    ///   that is newer than the stored one.
    /// - [`Ok(`](Ok)[`None`](None)[`)`](Ok), if the [`Reply`] contains an [`Ok(_)`](Ok) value that
    ///   is older than the stored one.
    pub fn update_reply<E>(&mut self, reply: Reply<T, E>) -> Result<Option<&T>, E> {
        let timestamp = reply.timestamp();
        let val = reply.into_args()?;
        Ok(self.update(val, timestamp))
    }

    /// Updates the value from an [`Event`], if its timestamp is newer.
    ///
    /// Returns [`Some(_)`](Some), if the value was updated.
    pub fn update_event(&mut self, ev: Event<T>) -> Option<&T> {
        let timestamp = ev.timestamp();
        let val = ev.into_args();
        self.update(val, timestamp)
    }
}

impl<T> Property<Option<T>> {
    /// Updates the value to [`Some(val)`](Some) if `timestamp` is newer.
    ///
    /// Returns [`Some(_)`](Some), if the value was updated.
    pub fn update_some(&mut self, val: T, timestamp: Instant) -> Option<&T> {
        if timestamp > self.timestamp {
            self.timestamp = timestamp;
            Some(self.val.insert(val))
        } else {
            None
        }
    }

    /// Updates the value to [`None`] if `timestamp` is newer.
    ///
    /// Returns [`true`] if the value was updated.
    pub fn update_none(&mut self, timestamp: Instant) -> bool {
        if timestamp > self.timestamp {
            self.timestamp = timestamp;
            true
        } else {
            false
        }
    }

    /// Updates the value from a [`Reply`] by wrapping it in [`Some(_)`](Some), if its timestamp is
    /// newer.
    ///
    /// Returns:
    /// - [`Err(_)`](Err), if the [`Reply`] contains an error.
    /// - [`Ok(`](Ok)[`Some(_)`](Some)[`)`](Ok), if the [`Reply`] contains an [`Ok(_)`](Ok) value
    ///   that is newer than the stored one.
    /// - [`Ok(`](Ok)[`None`](None)[`)`](Ok), if the [`Reply`] contains an [`Ok(_)`](Ok) value that
    ///   is older than the stored one.
    pub fn update_reply_some<E>(&mut self, reply: Reply<T, E>) -> Result<Option<&T>, E> {
        let timestamp = reply.timestamp();
        let val = reply.into_args()?;
        Ok(self.update_some(val, timestamp))
    }

    /// Updates the value from an [`Event`] by wrapping it in [`Some(_)`](Some), if its timestamp is
    /// newer.
    ///
    /// Returns [`Some(_)`](Some), if the value was updated.
    pub fn update_event_some(&mut self, ev: Event<T>) -> Option<&T> {
        let timestamp = ev.timestamp();
        let val = ev.into_args();
        self.update_some(val, timestamp)
    }
}

impl<T: PartialEq> Property<T> {
    /// Updates the value to `val` if it's different and `timestamp` is newer.
    ///
    /// Returns [`Some(_)`](Some), if the value was updated.
    ///
    /// If `timestamp` is newer, then the stored timestamp is always updated, regardless of `val`.
    pub fn check(&mut self, val: T, timestamp: Instant) -> Option<&T> {
        if timestamp > self.timestamp {
            self.timestamp = timestamp;

            if self.val != val {
                self.val = val;
                Some(&self.val)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Updates the value from a [`Reply`] if it's different and its timestamp is newer.
    ///
    /// Returns:
    /// - [`Err(_)`](Err), if the [`Reply`] contains an error.
    /// - [`Ok(`](Ok)[`Some(_)`](Some)[`)`](Ok), if the [`Reply`] contains an [`Ok(_)`](Ok) value
    ///   that is different and newer than the stored one.
    /// - [`Ok(`](Ok)[`None`](None)[`)`](Ok), if the [`Reply`] contains an [`Ok(_)`](Ok) value that
    ///   is either older than or equal to the stored one.
    ///
    /// If the [`Reply`s](Reply) `timestamp` is newer, then the stored timestamp is always updated,
    /// regardless of the [`Reply`s](Reply) value.
    pub fn check_reply<E>(&mut self, reply: Reply<T, E>) -> Result<Option<&T>, E> {
        let timestamp = reply.timestamp();
        let val = reply.into_args()?;
        Ok(self.check(val, timestamp))
    }

    /// Updates the value from an [`Event`], if it's different and its timestamp is newer.
    pub fn check_event(&mut self, ev: Event<T>) -> Option<&T> {
        let timestamp = ev.timestamp();
        let val = ev.into_args();
        self.check(val, timestamp)
    }
}

impl<T: PartialEq> Property<Option<T>> {
    /// Updates the value to [`Some(val)`](Some) if it's different and `timestamp` is newer.
    ///
    /// Returns [`Some(_)`](Some), if the value was updated.
    ///
    /// If `timestamp` is newer, then the stored timestamp is always updated, regardless of `val`.
    pub fn check_some(&mut self, val: T, timestamp: Instant) -> Option<&T> {
        if timestamp > self.timestamp {
            self.timestamp = timestamp;

            match self.val {
                Some(ref cur) if *cur == val => None,
                _ => Some(self.val.insert(val)),
            }
        } else {
            None
        }
    }
}

impl<T> Property<Option<T>> {
    /// Updates the value to [`None`] if it's currently [`Some(_)`](Some) and if `timestamp` is
    /// newer.
    ///
    /// Returns [`true`] if the value was updated.
    ///
    /// If `timestamp` is newer, then the stored timestamp is always updated, regardless of the
    /// current value.
    pub fn check_none(&mut self, timestamp: Instant) -> bool {
        if timestamp > self.timestamp {
            self.timestamp = timestamp;
            self.val.take().is_some()
        } else {
            false
        }
    }
}

impl<T: PartialEq> Property<Option<T>> {
    /// Updates the value from a [`Reply`] by wrapping it in [`Some(_)`](Some) if it's different and
    /// its timestamp is newer.
    ///
    /// Returns:
    /// - [`Err(_)`](Err), if the [`Reply`] contains an error.
    /// - [`Ok(`](Ok)[`Some(_)`](Some)[`)`](Ok), if the [`Reply`] contains an [`Ok(_)`](Ok) value
    ///   that is different and newer than the stored one.
    /// - [`Ok(`](Ok)[`None`](None)[`)`](Ok), if the [`Reply`] contains an [`Ok(_)`](Ok) value that
    ///   is either older than or equal to the stored one.
    ///
    /// If the [`Reply`s](Reply) `timestamp` is newer, then the stored timestamp is always updated,
    /// regardless of the [`Reply`s](Reply) value.
    pub fn check_reply_some<E>(&mut self, reply: Reply<T, E>) -> Result<Option<&T>, E> {
        let timestamp = reply.timestamp();
        let val = reply.into_args()?;
        Ok(self.check_some(val, timestamp))
    }

    /// Updates the value from an [`Event`] by wrapping it in [`Some(_)`](Some), if its different
    /// and its timestamp is newer.
    ///
    /// Returns [`Some(_)`](Some), if the value was updated.
    pub fn check_event_some(&mut self, ev: Event<T>) -> Option<&T> {
        let timestamp = ev.timestamp();
        let val = ev.into_args();
        self.check_some(val, timestamp)
    }
}

impl<T: Default> Default for Property<T> {
    fn default() -> Self {
        Self::new()
    }
}
