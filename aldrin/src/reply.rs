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
}
