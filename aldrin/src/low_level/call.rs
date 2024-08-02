use super::Promise;
use crate::core::{Deserialize, DeserializeError, SerializedValue, SerializedValueSlice};
use crate::error::Error;
use crate::handle::Handle;
use futures_channel::oneshot::Receiver;

/// Pending call.
#[derive(Debug)]
pub struct Call {
    id: u32,
    args: SerializedValue,
    promise: Promise,
}

impl Call {
    pub(crate) fn new(
        client: Handle,
        aborted: Receiver<()>,
        serial: u32,
        id: u32,
        args: SerializedValue,
    ) -> Self {
        Self {
            id,
            args,
            promise: Promise::new(client, aborted, serial),
        }
    }

    /// Returns a handle to the client that was used to create the call.
    pub fn client(&self) -> &Handle {
        self.promise.client()
    }

    /// Returns the call's function id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns a slice to the call's serialized arguments.
    pub fn args(&self) -> &SerializedValueSlice {
        &self.args
    }

    /// Deserializes the call's arguments.
    pub fn deserialize<T: Deserialize>(&self) -> Result<T, DeserializeError> {
        self.args.deserialize()
    }

    /// Converts this call into its promise object.
    pub fn into_promise(self) -> Promise {
        self.promise
    }

    /// Converts this call into its serialized arguments and a promise object.
    pub fn into_args_and_promise(self) -> (SerializedValue, Promise) {
        (self.args, self.promise)
    }

    /// Deserializes arguments and casts the promise to a specific set of result types.
    ///
    /// If deserialization fails, then the call will be replied using [`Promise::invalid_args`] and
    /// [`Error::InvalidArguments`] will be returned.
    pub fn deserialize_and_cast<Args, T, E>(
        self,
    ) -> Result<(Args, crate::promise::Promise<T, E>), Error>
    where
        Args: Deserialize,
        T: ?Sized,
        E: ?Sized,
    {
        match self.args.deserialize() {
            Ok(args) => Ok((args, self.promise.cast())),

            Err(e) => {
                let _ = self.promise.invalid_args();
                Err(Error::invalid_arguments(self.id, Some(e)))
            }
        }
    }
}
