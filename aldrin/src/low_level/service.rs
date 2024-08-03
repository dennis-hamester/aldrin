use super::Call;
#[cfg(feature = "introspection")]
use crate::core::introspection::Introspection;
use crate::core::{Serialize, SerializedValue, ServiceId, ServiceInfo, ServiceUuid, TypeId};
use crate::error::Error;
use crate::handle::Handle;
use crate::object::Object;
use futures_channel::mpsc::UnboundedReceiver;
use futures_channel::oneshot::Receiver;
use futures_core::stream::{FusedStream, Stream};
#[cfg(feature = "introspection")]
use std::borrow::Cow;
use std::future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Owned service.
#[derive(Debug)]
pub struct Service {
    id: ServiceId,
    info: ServiceInfo,
    client: Handle,
    calls: UnboundedReceiver<RawCall>,
}

impl Service {
    /// Creates a new service.
    pub async fn new(object: &Object, uuid: ServiceUuid, info: ServiceInfo) -> Result<Self, Error> {
        object.create_service(uuid, info).await
    }

    pub(crate) fn new_impl(
        id: ServiceId,
        info: ServiceInfo,
        client: Handle,
        calls: UnboundedReceiver<RawCall>,
    ) -> Self {
        Self {
            id,
            info,
            client,
            calls,
        }
    }

    /// Returns the id of the service.
    pub fn id(&self) -> ServiceId {
        self.id
    }

    /// Returns the version of the service.
    pub fn version(&self) -> u32 {
        self.info.version
    }

    /// Returns the type id of the service, if it was created with one.
    pub fn type_id(&self) -> Option<TypeId> {
        self.info.type_id
    }

    /// Queries the introspection for the service.
    #[cfg(feature = "introspection")]
    pub async fn query_introspection(&self) -> Result<Option<Cow<'static, Introspection>>, Error> {
        match self.info.type_id {
            Some(type_id) => self.client.query_introspection(type_id).await,
            None => Ok(None),
        }
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
                call.aborted,
                call.serial,
                call.function,
                call.args,
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
    pub fn emit<T>(&self, event: u32, args: &T) -> Result<(), Error>
    where
        T: Serialize + ?Sized,
    {
        self.client.emit_event(self.id, event, args)
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        self.client.destroy_service_now(self.id);
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

#[derive(Debug)]
pub(crate) struct RawCall {
    pub serial: u32,
    pub function: u32,
    pub args: SerializedValue,
    pub aborted: Receiver<()>,
}
