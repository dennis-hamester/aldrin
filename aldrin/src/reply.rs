use std::time::Instant;

/// Reply of a call.
#[derive(Debug, Clone)]
pub struct Reply<T, E> {
    id: u32,
    timestamp: Instant,
    args: Result<T, E>,
}

impl<T, E> Reply<T, E> {
    pub(crate) fn new(id: u32, timestamp: Instant, args: Result<T, E>) -> Self {
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

    /// Returns the reply's arguments as references.
    pub fn args(&self) -> Result<&T, &E> {
        self.args.as_ref()
    }

    /// Returns the reply's arguments as mutable references.
    pub fn args_mut(&mut self) -> Result<&mut T, &mut E> {
        self.args.as_mut()
    }

    /// Converts the reply to its arguments.
    pub fn into_args(self) -> Result<T, E> {
        self.args
    }

    /// Converts from `&Reply<T, E>` to `Reply<&T, &E>`.
    pub fn as_ref(&self) -> Reply<&T, &E> {
        Reply::new(self.id, self.timestamp, self.args.as_ref())
    }

    /// Converts from `&mut Reply<T, E>` to `Reply<&mut T, &mut E>`.
    pub fn as_mut(&mut self) -> Reply<&mut T, &mut E> {
        Reply::new(self.id, self.timestamp, self.args.as_mut())
    }

    /// Maps a `Reply<T, E>` to `Reply<U, F>` by applying a function to the arguments.
    pub fn map_args<O, U, F>(self, op: O) -> Reply<U, F>
    where
        O: FnOnce(Result<T, E>) -> Result<U, F>,
    {
        Reply::new(self.id, self.timestamp, op(self.args))
    }

    /// Maps a `Reply<T, E>` to `Reply<U, E>` by applying a function to the `Ok` arguments.
    pub fn map<F, U>(self, op: F) -> Reply<U, E>
    where
        F: FnOnce(T) -> U,
    {
        Reply::new(self.id, self.timestamp, self.args.map(op))
    }

    /// Maps a `Reply<T, E>` to `Reply<T, F>` by applying a function to the `Err` arguments.
    pub fn map_err<O, F>(self, op: O) -> Reply<T, F>
    where
        O: FnOnce(E) -> F,
    {
        Reply::new(self.id, self.timestamp, self.args.map_err(op))
    }
}
