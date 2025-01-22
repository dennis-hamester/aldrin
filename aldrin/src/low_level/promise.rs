use crate::core::message::CallFunctionResult;
use crate::core::Serialize;
use crate::error::Error;
use crate::handle::Handle;
use crate::Promise as HlPromise;
use futures_channel::oneshot::Receiver;
use futures_core::FusedFuture;
use std::future::{self, Future};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

/// Replies to a pending call.
#[derive(Debug)]
pub struct Promise {
    client: Option<Handle>,
    id: u32,
    timestamp: Instant,
    aborted: Receiver<()>,
    serial: u32,
}

impl Promise {
    pub(crate) fn new(
        client: Handle,
        id: u32,
        timestamp: Instant,
        aborted: Receiver<()>,
        serial: u32,
    ) -> Self {
        Self {
            client: Some(client),
            id,
            timestamp,
            aborted,
            serial,
        }
    }

    /// Returns a handle to the client that was used to create the promise.
    pub fn client(&self) -> &Handle {
        self.client.as_ref().unwrap()
    }

    /// Returns the call's function id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the timestamp when the call was received.
    pub fn timestamp(&self) -> Instant {
        self.timestamp
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

    pub(crate) fn invalid_function_ref(&mut self) {
        if let Some(client) = self.client.take() {
            let _ = client.function_call_reply(self.serial, CallFunctionResult::InvalidFunction);
        }
    }

    /// Signals that invalid arguments were passed to the function.
    pub fn invalid_args(mut self) -> Result<(), Error> {
        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, CallFunctionResult::InvalidArgs)
    }

    /// Returns whether the call was aborted by the caller.
    pub fn is_aborted(&mut self) -> bool {
        if self.aborted.is_terminated() {
            true
        } else {
            !matches!(self.aborted.try_recv(), Ok(None))
        }
    }

    /// Polls whether the call was aborted by the caller.
    pub fn poll_aborted(&mut self, cx: &mut Context) -> Poll<()> {
        if self.aborted.is_terminated() {
            Poll::Ready(())
        } else {
            Pin::new(&mut self.aborted).poll(cx).map(|_| ())
        }
    }

    /// Resolves if the call was aborted by the caller.
    pub async fn aborted(&mut self) {
        future::poll_fn(|cx| self.poll_aborted(cx)).await
    }
}

impl Drop for Promise {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            let _ = client.function_call_reply(self.serial, CallFunctionResult::Aborted);
        }
    }
}
