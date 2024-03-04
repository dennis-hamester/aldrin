use super::Promise;
use crate::core::{Deserialize, SerializedValue};
use crate::error::Error;
use crate::handle::Handle;
use futures_channel::oneshot::Receiver;

/// Pending call.
#[derive(Debug)]
pub struct Call {
    /// Id of the function that was called.
    pub function: u32,

    /// Arguments to the call.
    pub args: SerializedValue,

    /// Promise to reply to the call.
    pub promise: Promise,
}

impl Call {
    pub(crate) fn new(
        client: Handle,
        aborted: Receiver<()>,
        serial: u32,
        function: u32,
        args: SerializedValue,
    ) -> Self {
        Self {
            function,
            args,
            promise: Promise::new(client, aborted, serial),
        }
    }

    /// Returns a handle to the client that was used to create the call.
    pub fn client(&self) -> &Handle {
        self.promise.client()
    }

    /// Deserializes arguments and casts the promise to a specific set of result types.
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
                Err(Error::invalid_arguments(self.function, Some(e)))
            }
        }
    }
}
