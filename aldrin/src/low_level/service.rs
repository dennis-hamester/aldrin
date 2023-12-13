use super::Call;
use crate::core::{Serialize, ServiceId, ServiceUuid};
use crate::error::Error;
use crate::handle::Handle;
use crate::object::Object;
use crate::service::RawFunctionCall;
use futures_channel::mpsc::UnboundedReceiver;
use futures_core::stream::{FusedStream, Stream};
use std::future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Owned service.
#[derive(Debug)]
pub struct Service {
    id: ServiceId,
    version: u32,
    client: Handle,
    calls: UnboundedReceiver<RawFunctionCall>,
}

impl Service {
    /// Creates a new service.
    pub async fn new(_object: &Object, _uuid: ServiceUuid, _version: u32) -> Result<Self, Error> {
        todo!()
    }

    /// Returns the id of the service.
    pub fn id(&self) -> ServiceId {
        self.id
    }

    /// Returns the version of the service.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns a handle to the client that was used to create the service.
    pub fn client(&self) -> &Handle {
        &self.client
    }

    /// Destroys the service.
    ///
    /// If the [`Service`] has already been destroyed, then [`Error::InvalidService`] is returned.
    pub async fn destroy(&self) -> Result<(), Error> {
        self.client.destroy_service(self.id).await
    }

    /// Polls for the next call.
    pub fn poll_next_call(&mut self, cx: &mut Context) -> Poll<Option<Call>> {
        match Pin::new(&mut self.calls).poll_next(cx) {
            Poll::Ready(Some(call)) => Poll::Ready(Some(Call::new(
                self.client.clone(),
                call.serial,
                call.function,
                call.value,
            ))),

            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    /// Returns the next call.
    pub async fn next_call(&mut self) -> Option<Call> {
        future::poll_fn(|cx| self.poll_next_call(cx)).await
    }

    /// Emits an event.
    pub fn emit_event<T>(&self, event: u32, args: &T) -> Result<(), Error>
    where
        T: Serialize + ?Sized,
    {
        self.client.emit_event(self.id, event, args)
    }
}

impl Stream for Service {
    type Item = Call;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Call>> {
        self.poll_next_call(cx)
    }
}

impl FusedStream for Service {
    fn is_terminated(&self) -> bool {
        self.calls.is_terminated()
    }
}
