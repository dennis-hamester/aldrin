/// Reply of a call.
#[derive(Debug, Clone)]
pub struct Reply<T, E> {
    id: u32,
    args: Option<Result<T, E>>,
}

impl<T, E> Reply<T, E> {
    /// Returns the reply's function id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the reply's arguments as references.
    ///
    /// # Panics
    ///
    /// This function will panic if the arguments have been [taken](Self::take_args) out.
    pub fn args(&self) -> Result<&T, &E> {
        self.args
            .as_ref()
            .expect("args were already taken")
            .as_ref()
    }

    /// Returns the reply's arguments as mutable references.
    ///
    /// # Panics
    ///
    /// This function will panic if the arguments have been [taken](Self::take_args) out.
    pub fn args_mut(&mut self) -> Result<&mut T, &mut E> {
        self.args
            .as_mut()
            .expect("args were already taken")
            .as_mut()
    }

    /// Takes out the reply's arguments.
    ///
    /// # Panics
    ///
    /// This function will panic if the arguments have already been taken out.
    pub fn take_args(&mut self) -> Result<T, E> {
        self.args.take().expect("args were already taken")
    }

    /// Converts the reply to its arguments.
    ///
    /// # Panics
    ///
    /// This function will panic if the arguments have been [taken](Self::take_args) out.
    pub fn into_args(mut self) -> Result<T, E> {
        self.take_args()
    }

    /// Returns `true`, if the reply's arguments have not yet been taken out.
    pub fn has_args(&self) -> bool {
        self.args.is_some()
    }
}
