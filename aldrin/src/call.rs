use crate::core::{AsSerializeArg, Serialize, SerializeArg};
use crate::error::Error;
use crate::handle::Handle;
use crate::promise::Promise;
use std::fmt;
use std::task::{Context, Poll};

/// Pending call.
pub struct Call<Args, T: ?Sized, E: ?Sized> {
    id: u32,
    args: Option<Args>,
    promise: Option<Promise<T, E>>,
}

impl<Args, T: ?Sized, E: ?Sized> Call<Args, T, E> {
    /// Returns the call's function id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns a reference to the call's arguments.
    ///
    /// # Panics
    ///
    /// This function will panic if the arguments have been [taken](Self::take_args) out.
    pub fn args(&self) -> &Args {
        self.args.as_ref().expect("args were already taken")
    }

    /// Returns a mutable reference to the call's arguments.
    ///
    /// # Panics
    ///
    /// This function will panic if the arguments have been [taken](Self::take_args) out.
    pub fn args_mut(&mut self) -> &mut Args {
        self.args.as_mut().expect("args were already taken")
    }

    /// Takes out the call's arguments.
    ///
    /// # Panics
    ///
    /// This function will panic if the arguments have already been taken out.
    pub fn take_args(&mut self) -> Args {
        self.args.take().expect("args were already taken")
    }

    /// Returns `true`, if the call's arguments have not yet been taken out.
    pub fn has_args(&self) -> bool {
        self.args.is_some()
    }

    /// Returns a reference to the call's promise object.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn promise(&self) -> &Promise<T, E> {
        self.promise.as_ref().expect("promise was already taken")
    }

    /// Returns a mutable reference to the call's promise object.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn promise_mut(&mut self) -> &mut Promise<T, E> {
        self.promise.as_mut().expect("promise was already taken")
    }

    /// Takes out the call's promise object.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has already been taken out, either by this function
    /// or by one of the functions, which implicitly consume the promise.
    pub fn take_promise(&mut self) -> Promise<T, E> {
        self.promise.take().expect("promise was already taken")
    }

    /// Returns `true`, if the call's promise object has not yet been taken out.
    pub fn has_promise(&self) -> bool {
        self.promise.is_some()
    }

    /// Casts the call to a different result type.
    pub fn cast<T2: ?Sized, E2: ?Sized>(self) -> Call<Args, T2, E2> {
        Call {
            id: self.id,
            args: self.args,
            promise: self.promise.map(Promise::cast),
        }
    }

    /// Returns a handle to the client that was used to create the call.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn client(&self) -> &Handle {
        self.promise().client()
    }

    /// Aborts the call.
    ///
    /// The caller will be notified that the call was aborted.
    ///
    /// This function consumes the promise.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn abort(&mut self) -> Result<(), Error> {
        self.take_promise().abort()
    }

    /// Signals that an invalid function was called.
    ///
    /// This function consumes the promise.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn invalid_function(&mut self) -> Result<(), Error> {
        self.take_promise().invalid_function()
    }

    /// Signals that invalid arguments were passed to the function.
    ///
    /// This function consumes the promise.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn invalid_args(&mut self) -> Result<(), Error> {
        self.take_promise().invalid_args()
    }

    /// Returns whether the call was aborted by the caller.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn is_aborted(&mut self) -> bool {
        self.promise_mut().is_aborted()
    }

    /// Polls whether the call was aborted by the caller.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn poll_aborted(&mut self, cx: &mut Context) -> Poll<()> {
        self.promise_mut().poll_aborted(cx)
    }

    /// Resolves if the call was aborted by the caller.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub async fn aborted(&mut self) {
        self.promise_mut().aborted().await
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: AsSerializeArg + ?Sized,
    E: ?Sized,
{
    /// Signals that the call was successful.
    ///
    /// This function consumes the promise.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn ok(&mut self, value: SerializeArg<T>) -> Result<(), Error> {
        self.take_promise().ok(value)
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: Serialize + ?Sized,
    E: ?Sized,
{
    /// Signals that the call was successful.
    ///
    /// This function consumes the promise.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn ok_ref(&mut self, value: &T) -> Result<(), Error> {
        self.take_promise().ok_ref(value)
    }
}

impl<Args, E: ?Sized> Call<Args, (), E> {
    /// Signals that the call was successful without returning a value.
    ///
    /// This function consumes the promise.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn done(&mut self) -> Result<(), Error> {
        self.take_promise().done()
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: ?Sized,
    E: AsSerializeArg + ?Sized,
{
    /// Signals that the call failed.
    ///
    /// This function consumes the promise.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn err(&mut self, value: SerializeArg<E>) -> Result<(), Error> {
        self.take_promise().err(value)
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: ?Sized,
    E: Serialize + ?Sized,
{
    /// Signals that the call failed.
    ///
    /// This function consumes the promise.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn err_ref(&mut self, value: &E) -> Result<(), Error> {
        self.take_promise().err_ref(value)
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: AsSerializeArg + ?Sized,
    E: AsSerializeArg + ?Sized,
{
    /// Sets the call's reply.
    ///
    /// This function consumes the promise.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn set(&mut self, res: Result<SerializeArg<T>, SerializeArg<E>>) -> Result<(), Error> {
        self.take_promise().set(res)
    }
}

impl<Args, T, E> Call<Args, T, E>
where
    T: Serialize + ?Sized,
    E: Serialize + ?Sized,
{
    /// Sets the call's reply.
    ///
    /// This function consumes the promise.
    ///
    /// # Panics
    ///
    /// This function will panic if the promise has been taken out, either
    /// [explicitly](Self::take_promise) or by one of the functions, which implicitly consume the
    /// promise.
    pub fn set_ref(&mut self, res: Result<&T, &E>) -> Result<(), Error> {
        self.take_promise().set_ref(res)
    }
}

impl<Args, T, E> fmt::Debug for Call<Args, T, E>
where
    Args: fmt::Debug,
    T: ?Sized,
    E: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Call")
            .field("id", &self.id)
            .field("args", &self.args)
            .field("promise", &self.promise)
            .finish()
    }
}
