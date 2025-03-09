use crate::{Error, Handle};
use aldrin_core::message::CallFunctionResult;
use aldrin_core::tags::{PrimaryTag, Tag};
use aldrin_core::{Serialize, SerializedValue};
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
    version: Option<u32>,
    timestamp: Instant,
    aborted: Receiver<()>,
    serial: u32,
}

impl Promise {
    pub(crate) fn new(
        client: Handle,
        id: u32,
        version: Option<u32>,
        timestamp: Instant,
        aborted: Receiver<()>,
        serial: u32,
    ) -> Self {
        Self {
            client: Some(client),
            id,
            version,
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

    /// Returns the version number used to make the call, if any.
    pub fn version(&self) -> Option<u32> {
        self.version
    }

    /// Returns the timestamp when the call was received.
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    /// Casts the promise to a specific set of result types.
    pub fn cast<T, E>(self) -> crate::promise::Promise<T, E> {
        crate::Promise::new(self)
    }

    /// Sets the call's reply.
    pub fn set_as<T, U, E, F>(self, res: Result<U, F>) -> Result<(), Error>
    where
        T: Tag,
        U: Serialize<T>,
        E: Tag,
        F: Serialize<E>,
    {
        match res {
            Ok(value) => self.ok_as(value),
            Err(value) => self.err_as(value),
        }
    }

    /// Sets the call's reply.
    pub fn set<T, E>(self, res: Result<T, E>) -> Result<(), Error>
    where
        T: PrimaryTag + Serialize<T::Tag>,
        E: PrimaryTag + Serialize<E::Tag>,
    {
        self.set_as(res)
    }

    /// Signals that the call was successful.
    pub fn ok_as<T: Tag, U: Serialize<T>>(mut self, value: U) -> Result<(), Error> {
        let res = CallFunctionResult::Ok(SerializedValue::serialize_as(value)?);

        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, res)
    }

    /// Signals that the call was successful.
    pub fn ok<T: PrimaryTag + Serialize<T::Tag>>(self, value: T) -> Result<(), Error> {
        self.ok_as(value)
    }

    /// Signals that the call was successful without returning a value.
    pub fn done(mut self) -> Result<(), Error> {
        let res = CallFunctionResult::Ok(SerializedValue::serialize(())?);

        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, res)
    }

    /// Signals that the call failed.
    pub fn err_as<E: Tag, F: Serialize<E>>(mut self, value: F) -> Result<(), Error> {
        let res = CallFunctionResult::Err(SerializedValue::serialize_as(value)?);

        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, res)
    }

    /// Signals that the call failed.
    pub fn err<E: PrimaryTag + Serialize<E::Tag>>(self, value: E) -> Result<(), Error> {
        self.err_as(value)
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
