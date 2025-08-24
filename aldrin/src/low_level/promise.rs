use crate::{Error, Handle};
use aldrin_core::message::CallFunctionResult;
use aldrin_core::tags::Tag;
use aldrin_core::{Serialize, SerializePrimary, SerializedValue, ServiceId};
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
    service: ServiceId,
}

impl Promise {
    pub(crate) fn new(
        client: Handle,
        id: u32,
        version: Option<u32>,
        timestamp: Instant,
        aborted: Receiver<()>,
        serial: u32,
        service: ServiceId,
    ) -> Self {
        Self {
            client: Some(client),
            id,
            version,
            timestamp,
            aborted,
            serial,
            service,
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

    /// Returns the id of the service that the call was received for.
    pub fn service(&self) -> ServiceId {
        self.service
    }

    /// Casts the promise to a specific set of result types.
    pub fn cast<T, E>(self) -> crate::promise::Promise<T, E> {
        crate::Promise::new(self)
    }

    /// Sets the call's reply.
    pub fn set_as<T, E>(
        self,
        res: Result<impl Serialize<T>, impl Serialize<E>>,
    ) -> Result<(), Error>
    where
        T: Tag,
        E: Tag,
    {
        match res {
            Ok(value) => self.ok_as(value),
            Err(value) => self.err_as(value),
        }
    }

    /// Sets the call's reply.
    pub fn set(
        self,
        res: Result<impl SerializePrimary, impl SerializePrimary>,
    ) -> Result<(), Error> {
        self.set_as(res)
    }

    /// Signals that the call was successful.
    pub fn ok_as<T: Tag>(mut self, value: impl Serialize<T>) -> Result<(), Error> {
        let res = CallFunctionResult::Ok(SerializedValue::serialize_as(value)?);

        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, res)
    }

    /// Signals that the call was successful.
    pub fn ok(self, value: impl SerializePrimary) -> Result<(), Error> {
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
    pub fn err_as<E: Tag>(mut self, value: impl Serialize<E>) -> Result<(), Error> {
        let res = CallFunctionResult::Err(SerializedValue::serialize_as(value)?);

        self.client
            .take()
            .unwrap()
            .function_call_reply(self.serial, res)
    }

    /// Signals that the call failed.
    pub fn err(self, value: impl SerializePrimary) -> Result<(), Error> {
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
