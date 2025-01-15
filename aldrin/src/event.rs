/// Event emitted by a service.
#[derive(Debug, Clone)]
pub struct Event<T> {
    id: u32,
    args: Option<T>,
}

impl<T> Event<T> {
    pub(crate) fn new(id: u32, args: T) -> Self {
        Self {
            id,
            args: Some(args),
        }
    }

    /// Returns the event's id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns a reference to the event's arguments.
    ///
    /// # Panics
    ///
    /// This function will panic if the arguments have been [taken](Self::take_args) out.
    pub fn args(&self) -> &T {
        self.args.as_ref().expect("args were already taken")
    }

    /// Returns a mutable reference to the event's arguments.
    ///
    /// # Panics
    ///
    /// This function will panic if the arguments have been [taken](Self::take_args) out.
    pub fn args_mut(&mut self) -> &mut T {
        self.args.as_mut().expect("args were already taken")
    }

    /// Takes out the event's arguments.
    ///
    /// # Panics
    ///
    /// This function will panic if the arguments have already been taken out.
    pub fn take_args(&mut self) -> T {
        self.args.take().expect("args were already taken")
    }

    /// Converts the event to its arguments.
    ///
    /// # Panics
    ///
    /// This function will panic if the arguments have been [taken](Self::take_args) out.
    pub fn into_args(mut self) -> T {
        self.take_args()
    }

    /// Returns `true`, if the event's arguments have not yet been taken out.
    pub fn has_args(&self) -> bool {
        self.args.is_some()
    }
}
