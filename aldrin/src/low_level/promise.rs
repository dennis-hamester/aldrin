use crate::core::message::CallFunctionResult;
use crate::core::Serialize;
use crate::error::Error;
use crate::handle::Handle;
use crate::Promise as HlPromise;

/// Replies to a pending call.
#[derive(Debug)]
pub struct Promise {
    client: Option<Handle>,
    serial: u32,
}

impl Promise {
    pub(crate) fn new(client: Handle, serial: u32) -> Self {
        Self {
            client: Some(client),
            serial,
        }
    }

    /// Casts the promise to a specific set of result types.
    pub fn cast<T: ?Sized, E: ?Sized>(self) -> crate::promise::Promise<T, E> {
        HlPromise::new(self)
    }

    /// Sets the call's reply.
    pub fn set<T, E>(self, res: Result<&T, &E>) -> Result<(), Error>
    where
        T: Serialize + ?Sized,
        E: Serialize + ?Sized,
    {
        match res {
            Ok(value) => self.ok(value),
            Err(value) => self.err(value),
        }
    }

    /// Signals that the call was successful.
    pub fn ok<T>(mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize + ?Sized,
    {
        let res = CallFunctionResult::ok_with_serialize_value(value)?;

        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, res)
    }

    /// Signals that the call was successful without returning a value.
    pub fn done(mut self) -> Result<(), Error> {
        let res = CallFunctionResult::ok_with_serialize_value(&())?;

        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, res)
    }

    /// Signals that the call failed.
    pub fn err<E>(mut self, value: &E) -> Result<(), Error>
    where
        E: Serialize + ?Sized,
    {
        let res = CallFunctionResult::err_with_serialize_value(value)?;

        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, res)
    }

    /// Aborts the call.
    ///
    /// The caller will be notified that the call was aborted.
    pub fn abort(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::Aborted)
    }

    /// Signals that an invalid function was called.
    pub fn invalid_function(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::InvalidFunction)
    }

    /// Signals that invalid arguments were passed to the function.
    pub fn invalid_args(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::InvalidArgs)
    }
}

impl Drop for Promise {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            let _ = client.function_call_reply(self.serial, CallFunctionResult::Aborted);
        }
    }
}
