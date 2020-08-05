use super::{
    EmitEventRequest, Error, Events, EventsId, EventsRequest, Object, ObjectCookie, ObjectEvent,
    ObjectId, ObjectUuid, Objects, Request, Service, ServiceCookie, ServiceEvent, ServiceId,
    ServiceUuid, Services, SubscribeEventRequest, SubscribeMode, UnsubscribeEventRequest,
};
use aldrin_proto::*;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_channel::oneshot;
use futures_util::stream::StreamExt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Handle to a client.
///
/// After connecting a [`Client`](crate::Client) to a broker, [`Handle`s](Handle) are used to
/// interact with it. The first [`Handle`] can be acquired with
/// [`Client::handle`](crate::Client::handle). After that, [`Handle`s](Handle) can be cloned
/// cheaply.
///
/// The [`Client`](crate::Client) will automatically shut down when the last [`Handle`] has been
/// dropped.
///
/// # Examples
///
/// ```
/// use aldrin_client::Client;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let broker = aldrin_broker::Broker::new();
/// # let handle = broker.handle().clone();
/// # tokio::spawn(broker.run());
/// # let (async_transport, t2) = aldrin_util::channel::unbounded();
/// # let conn = tokio::spawn(async move { handle.add_connection(t2).await });
/// // Connect to the broker:
/// let client = Client::connect(async_transport).await?;
///
/// // Acquire the first handle:
/// let handle = client.handle().clone();
///
/// // Run the client, which consumes it and leaves only the handle for interacting with it:
/// tokio::spawn(client.run());
/// # tokio::spawn(conn.await??.run());
///
/// // Handles are cheap to clone:
/// let handle2 = handle.clone();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Handle {
    send: UnboundedSender<Request>,
}

impl Handle {
    pub(crate) fn new(send: UnboundedSender<Request>) -> Self {
        Handle { send }
    }

    /// Shuts down the client.
    ///
    /// Shutdown happens asynchronously, in the sense that when this function returns, the
    /// [`Client`](crate::Client) has only been requested to shut down and not yet necessarily done
    /// so. As soon as [`Client::run`](crate::Client::run) returns, it has fully shut down.
    ///
    /// If the [`Client`](crate::Client) has already shut down (due to any reason), this function
    /// will not treat that as an error. This is different than most other functions, which would
    /// return [`Error::ClientShutdown`] instead.
    pub fn shutdown(&self) {
        self.send.unbounded_send(Request::Shutdown).ok();
    }

    /// Creates a new object on the bus.
    ///
    /// The `uuid` must not yet exists on the bus, or else [`Error::DuplicateObject`] will be
    /// returned. Use [`ObjectUuid::new_v4`] to create a new random v4 UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::{Error, ObjectUuid};
    ///
    /// // 6173e119-8066-4776-989b-145a5f16ed4c
    /// const OBJECT2_UUID: ObjectUuid = ObjectUuid::from_u128(0x6173e11980664776989b145a5f16ed4c);
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_broker::Broker::new();
    /// # let handle = broker.handle().clone();
    /// # tokio::spawn(broker.run());
    /// # let (async_transport, t2) = aldrin_util::channel::unbounded();
    /// # let conn = tokio::spawn(async move { handle.add_connection(t2).await });
    /// # let client = aldrin_client::Client::connect(async_transport).await?;
    /// # let handle = client.handle().clone();
    /// # tokio::spawn(client.run());
    /// # tokio::spawn(conn.await??.run());
    /// // Create an object with a random UUID:
    /// let object1 = handle.create_object(ObjectUuid::new_v4()).await?;
    ///
    /// // Create an object with a fixed UUID:
    /// let object2 = handle.create_object(OBJECT2_UUID).await?;
    ///
    /// // Using the same UUID again will cause an error:
    /// assert_eq!(
    ///     handle.create_object(OBJECT2_UUID).await.unwrap_err(),
    ///     Error::DuplicateObject(OBJECT2_UUID)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_object(&self, uuid: ObjectUuid) -> Result<Object, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .unbounded_send(Request::CreateObject(uuid, res_send))
            .map_err(|_| Error::ClientShutdown)?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            CreateObjectResult::Ok(cookie) => Ok(Object::new(
                ObjectId::new(uuid, ObjectCookie(cookie)),
                self.clone(),
            )),
            CreateObjectResult::DuplicateObject => Err(Error::DuplicateObject(uuid)),
        }
    }

    pub(crate) async fn destroy_object(&self, id: ObjectId) -> Result<(), Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .unbounded_send(Request::DestroyObject(id.cookie, res_send))
            .map_err(|_| Error::ClientShutdown)?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyObjectResult::Ok => Ok(()),
            DestroyObjectResult::InvalidObject => Err(Error::InvalidObject(id)),
            DestroyObjectResult::ForeignObject => unreachable!(),
        }
    }

    pub(crate) fn destroy_object_now(&self, cookie: ObjectCookie) {
        let (res_send, _) = oneshot::channel();
        self.send
            .unbounded_send(Request::DestroyObject(cookie, res_send))
            .ok();
    }

    /// Subscribes to a stream of object creation and destruction events.
    ///
    /// An [`Objects`] stream can be used to discover and track [`Object`s](Object) on the bus. The
    /// `mode` parameter decides whether the stream will include only current, only new or all
    /// [`Object`s](Object).
    ///
    /// See [`Objects`] for more information and usage examples.
    pub fn objects(&self, mode: SubscribeMode) -> Result<Objects, Error> {
        let (ev_send, ev_recv) = unbounded();
        self.send
            .unbounded_send(Request::SubscribeObjects(ev_send, mode))
            .map_err(|_| Error::ClientShutdown)?;
        Ok(Objects::new(ev_recv))
    }

    pub(crate) async fn create_service(
        &self,
        object_id: ObjectId,
        service_uuid: ServiceUuid,
    ) -> Result<Service, Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .unbounded_send(Request::CreateService(
                object_id.cookie,
                service_uuid,
                res_send,
            ))
            .map_err(|_| Error::ClientShutdown)?;
        let (res, recv) = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match res {
            CreateServiceResult::Ok(cookie) => Ok(Service::new(
                ServiceId::new(object_id, service_uuid, ServiceCookie(cookie)),
                self.clone(),
                recv.unwrap(),
            )),
            CreateServiceResult::DuplicateService => {
                Err(Error::DuplicateService(object_id, service_uuid))
            }
            CreateServiceResult::InvalidObject => Err(Error::InvalidObject(object_id)),
            CreateServiceResult::ForeignObject => unreachable!(),
        }
    }

    pub(crate) async fn destroy_service(&self, id: ServiceId) -> Result<(), Error> {
        let (res_send, res_reply) = oneshot::channel();
        self.send
            .unbounded_send(Request::DestroyService(id.cookie, res_send))
            .map_err(|_| Error::ClientShutdown)?;
        let reply = res_reply.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            DestroyServiceResult::Ok => Ok(()),
            DestroyServiceResult::InvalidService => Err(Error::InvalidService(id)),
            DestroyServiceResult::ForeignObject => unreachable!(),
        }
    }

    pub(crate) fn destroy_service_now(&self, cookie: ServiceCookie) {
        let (res_send, _) = oneshot::channel();
        self.send
            .unbounded_send(Request::DestroyService(cookie, res_send))
            .ok();
    }

    /// Subscribes to a stream of service creation and destruction events.
    ///
    /// A [`Services`] stream can be used to discover and track [`Service`s](Service) on the
    /// bus. The `mode` parameter decides whether the stream will include only current, only new or
    /// all [`Service`s](Service).
    ///
    /// See [`Services`] for more information and usage examples.
    pub fn services(&self, mode: SubscribeMode) -> Result<Services, Error> {
        let (ev_send, ev_recv) = unbounded();
        self.send
            .unbounded_send(Request::SubscribeServices(ev_send, mode))
            .map_err(|_| Error::ClientShutdown)?;
        Ok(Services::new(ev_recv))
    }

    /// Calls a function on a service.
    ///
    /// The function with id `function` will be called with the arguments `args` on the service
    /// identified by `service_id`.
    ///
    /// The returned value of type [`CallFunctionFuture`] is a future which will resolve to the
    /// result of the function call.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_proto::{FromValue, IntoValue};
    /// # use futures::stream::StreamExt;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_broker::Broker::new();
    /// # let handle = broker.handle().clone();
    /// # tokio::spawn(broker.run());
    /// # let (async_transport, t2) = aldrin_util::channel::unbounded();
    /// # let conn = tokio::spawn(async move { handle.add_connection(t2).await });
    /// # let client = aldrin_client::Client::connect(async_transport).await?;
    /// # let handle = client.handle().clone();
    /// # tokio::spawn(client.run());
    /// # tokio::spawn(conn.await??.run());
    /// # let obj = handle.create_object(aldrin_client::ObjectUuid::new_v4()).await?;
    /// # let mut svc = obj.create_service(aldrin_client::ServiceUuid(uuid::Uuid::new_v4())).await?;
    /// # let service_id = svc.id();
    /// // Call function 1 with "1 + 2 = ?" as the argument.
    /// let result = handle.call_function(service_id, 1, "1 + 2 = ?".into_value())?;
    /// # svc.next().await.unwrap().reply.ok(3u32.into_value())?;
    ///
    /// // Await the result. The `?` here checks for errors on the protocol level, such as a
    /// // intermediate shutdown, or whether the function call was aborted by the callee.
    /// let result = result.await?;
    ///
    /// // Now, result is of type `Result<Value, Value>`, directly representing the return value of
    /// // the function call.
    /// match result {
    ///     Ok(ok) => assert_eq!(u32::from_value(ok)?, 3),
    ///     Err(err) => panic!("Function call failed: {:?}.", err),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn call_function(
        &self,
        service_id: ServiceId,
        function: u32,
        args: Value,
    ) -> Result<CallFunctionFuture, Error> {
        let (send, recv) = oneshot::channel();
        self.send
            .unbounded_send(Request::CallFunction(
                service_id.cookie,
                function,
                args,
                send,
            ))
            .map_err(|_| Error::ClientShutdown)?;
        Ok(CallFunctionFuture {
            recv,
            service_id,
            function,
        })
    }

    pub(crate) fn function_call_reply(
        &self,
        serial: u32,
        result: CallFunctionResult,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(Request::FunctionCallReply(serial, result))
            .map_err(|_| Error::ClientShutdown)
    }

    pub(crate) fn abort_function_call_now(&self, serial: u32) {
        self.send
            .unbounded_send(Request::FunctionCallReply(
                serial,
                CallFunctionResult::Aborted,
            ))
            .ok();
    }

    /// Creates an Events object used to subscribe to service events.
    ///
    /// See [`Events`] for more information and usage examples.
    pub fn events(&self) -> Events {
        Events::new(self.clone())
    }

    pub(crate) async fn subscribe_event(
        &self,
        events_id: EventsId,
        service_id: ServiceId,
        id: u32,
        sender: UnboundedSender<EventsRequest>,
    ) -> Result<(), Error> {
        let (rep_send, rep_recv) = oneshot::channel();
        self.send
            .unbounded_send(Request::SubscribeEvent(SubscribeEventRequest {
                events_id,
                service_cookie: service_id.cookie,
                id,
                sender,
                reply: rep_send,
            }))
            .map_err(|_| Error::ClientShutdown)?;
        let reply = rep_recv.await.map_err(|_| Error::ClientShutdown)?;
        match reply {
            SubscribeEventResult::Ok => Ok(()),
            SubscribeEventResult::InvalidService => Err(Error::InvalidService(service_id)),
        }
    }

    pub(crate) fn unsubscribe_event(
        &self,
        events_id: EventsId,
        service_id: ServiceId,
        id: u32,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(Request::UnsubscribeEvent(UnsubscribeEventRequest {
                events_id,
                service_cookie: service_id.cookie,
                id,
            }))
            .map_err(|_| Error::ClientShutdown)
    }

    /// Emits an events to subscribed clients.
    ///
    /// The event with the id `event` of the service identified by `service_id` will be emitted with
    /// the arguments `args` to all subscribed clients.
    ///
    /// Use [`Handle::events`] to subscribe to events.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::IntoValue;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_broker::Broker::new();
    /// # let handle = broker.handle().clone();
    /// # tokio::spawn(broker.run());
    /// # let (async_transport, t2) = aldrin_util::channel::unbounded();
    /// # let conn = tokio::spawn(async move { handle.add_connection(t2).await });
    /// # let client = aldrin_client::Client::connect(async_transport).await?;
    /// # let handle = client.handle().clone();
    /// # tokio::spawn(client.run());
    /// # tokio::spawn(conn.await??.run());
    /// # let obj = handle.create_object(aldrin_client::ObjectUuid::new_v4()).await?;
    /// # let mut svc = obj.create_service(aldrin_client::ServiceUuid(uuid::Uuid::new_v4())).await?;
    /// # let service_id = svc.id();
    /// // Emit event 1 with argument "Hello, world!":
    /// handle.emit_event(service_id, 1, "Hello, world!".into_value()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn emit_event(
        &self,
        service_id: ServiceId,
        event: u32,
        args: Value,
    ) -> Result<(), Error> {
        self.send
            .unbounded_send(Request::EmitEvent(EmitEventRequest {
                service_cookie: service_id.cookie,
                event,
                args,
            }))
            .map_err(|_| Error::ClientShutdown)
    }

    /// Finds an object on the bus.
    ///
    /// Finds an [`Object`] with the [`ObjectUuid`] `uuid` and returns its [`ObjectId`]. If the
    /// [`Object`] does not currently exist on the bus, `None` will be returned. Use
    /// [`wait_for_object`](Handle::wait_for_object) to wait indefinitely until an [`Object`]
    /// appears.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::ObjectUuid;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_broker::Broker::new();
    /// # let handle = broker.handle().clone();
    /// # tokio::spawn(broker.run());
    /// # let (async_transport, t2) = aldrin_util::channel::unbounded();
    /// # let conn = tokio::spawn(async move { handle.add_connection(t2).await });
    /// # let client = aldrin_client::Client::connect(async_transport).await?;
    /// # let handle = client.handle().clone();
    /// # tokio::spawn(client.run());
    /// # tokio::spawn(conn.await??.run());
    /// // Create an object:
    /// let object_uuid = ObjectUuid::new_v4();
    /// let object = handle.create_object(object_uuid).await?;
    ///
    /// // Find the object:
    /// let object_id = handle.find_object(object_uuid).await?.expect("not found");
    /// assert_eq!(object_id, object.id());
    ///
    /// // Searching for a non-existent object will yield None:
    /// let non_existent = handle.find_object(ObjectUuid::new_v4()).await?;
    /// assert!(non_existent.is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_object(&self, uuid: ObjectUuid) -> Result<Option<ObjectId>, Error> {
        let mut objects = self.objects(SubscribeMode::CurrentOnly)?;

        while let Some(ev) = objects.next().await {
            match ev {
                ObjectEvent::Created(id) if id.uuid == uuid => return Ok(Some(id)),
                _ => {}
            }
        }

        Ok(None)
    }

    /// Waits for an object on the bus.
    ///
    /// Waits for an [`Object`] with the [`ObjectUuid`] `uuid` and returns its [`ObjectId`]. This
    /// function will wait indefinitely until the [`Object`] appears. Use
    /// [`find_object`](Handle::find_object) search only the currently existing [`Object`s](Object).
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::ObjectUuid;
    /// use std::time::Duration;
    /// use tokio::time;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_broker::Broker::new();
    /// # let handle = broker.handle().clone();
    /// # tokio::spawn(broker.run());
    /// # let (async_transport, t2) = aldrin_util::channel::unbounded();
    /// # let conn = tokio::spawn(async move { handle.add_connection(t2).await });
    /// # let client = aldrin_client::Client::connect(async_transport).await?;
    /// # let handle = client.handle().clone();
    /// # tokio::spawn(client.run());
    /// # tokio::spawn(conn.await??.run());
    /// let object_uuid = ObjectUuid::new_v4();
    ///
    /// // Wait until an object the above UUID appears.
    /// // Awaiting the future now would block indefinitely though.
    /// let object_id = handle.wait_for_object(object_uuid);
    ///
    /// // Create the object:
    /// let object = handle.create_object(object_uuid).await?;
    ///
    /// // Now the future will resolve:
    /// let object_id = object_id.await?;
    /// assert_eq!(object_id, object.id());
    ///
    /// let non_existent = time::timeout(
    ///     Duration::from_millis(500),
    ///     handle.wait_for_object(ObjectUuid::new_v4()),
    /// )
    /// .await;
    /// assert!(non_existent.is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_object(&self, uuid: ObjectUuid) -> Result<ObjectId, Error> {
        let mut objects = self.objects(SubscribeMode::All)?;

        while let Some(ev) = objects.next().await {
            match ev {
                ObjectEvent::Created(id) if id.uuid == uuid => return Ok(id),
                _ => {}
            }
        }

        Err(Error::ClientShutdown)
    }

    /// Finds an object with a specific service on the bus.
    ///
    /// Finds an [`Object`], which has a [`Service`] with UUID `service_uuid`. The [`ObjectUuid`]
    /// can be optionally specified as well with `object_uuid`. If it is not specified, then it is
    /// unspecified which [`Object`] the returned [`ServiceId`] belongs to, if any. This function
    /// considers only [`Service`s](Service), which currently exist on the bus. Use
    /// [`wait_for_service`](Handle::wait_for_service) to wait indefinitely for a [`Service`]
    /// matching the criteria.
    ///
    /// If you need more control or even a stream of all [`Object`s](Object) implementing a
    /// particular [`Service`], then use [`services`](Handle::services) instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_client::{ObjectUuid, ServiceUuid};
    /// use uuid::Uuid;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let broker = aldrin_broker::Broker::new();
    /// # let handle = broker.handle().clone();
    /// # tokio::spawn(broker.run());
    /// # let (async_transport, t2) = aldrin_util::channel::unbounded();
    /// # let conn = tokio::spawn(async move { handle.add_connection(t2).await });
    /// # let client = aldrin_client::Client::connect(async_transport).await?;
    /// # let handle = client.handle().clone();
    /// # tokio::spawn(client.run());
    /// # tokio::spawn(conn.await??.run());
    /// // Create an object and service:
    /// let object_uuid = ObjectUuid::new_v4();
    /// let object = handle.create_object(object_uuid).await?;
    /// let service_uuid = ServiceUuid(Uuid::new_v4());
    /// let service = object.create_service(service_uuid).await?;
    ///
    /// // Find a service without specifying an object UUID:
    /// let service_id = handle
    ///     .find_service(service_uuid, None)
    ///     .await?
    ///     .expect("service not found");
    /// // It could be any object (and service cookie), but the service UUID will match:
    /// assert_eq!(service_id.uuid, service.id().uuid);
    ///
    /// // Find a service on a specific object:
    /// let service_id = handle
    ///     .find_service(service_uuid, Some(object_uuid))
    ///     .await?
    ///     .expect("service not found");
    /// // The service id will match:
    /// assert_eq!(service_id, service.id());
    ///
    /// // Searching for a non-existent service yields None:
    /// let non_existent = handle
    ///     .find_service(ServiceUuid(Uuid::new_v4()), None)
    ///     .await?;
    /// assert!(non_existent.is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_service(
        &self,
        service_uuid: ServiceUuid,
        object_uuid: Option<ObjectUuid>,
    ) -> Result<Option<ServiceId>, Error> {
        let mut services = self.services(SubscribeMode::CurrentOnly)?;

        while let Some(ev) = services.next().await {
            let id = match ev {
                ServiceEvent::Created(id) if id.uuid == service_uuid => id,
                _ => continue,
            };

            if let Some(object_uuid) = object_uuid {
                if id.object_id.uuid != object_uuid {
                    continue;
                }
            }

            return Ok(Some(id));
        }

        Ok(None)
    }
}

/// Future to await the result of a function call.
///
/// The future resolves to the type `Result<Result<`[`Value`]`, `[`Value`]`>, `[`Error`]`>`. The
/// outer `Result<_, `[`Error`]`>` represents the success or failure on the protocol and client
/// library level. The inner `Result<`[`Value`]`, `[`Value`]`>` represents the actual return value
/// of the function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct CallFunctionFuture {
    recv: oneshot::Receiver<CallFunctionResult>,
    service_id: ServiceId,
    function: u32,
}

impl Future for CallFunctionFuture {
    type Output = Result<Result<Value, Value>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let res = match Pin::new(&mut self.recv).poll(cx) {
            Poll::Ready(Ok(res)) => res,
            Poll::Ready(Err(_)) => return Poll::Ready(Err(Error::ClientShutdown)),
            Poll::Pending => return Poll::Pending,
        };

        Poll::Ready(match res {
            CallFunctionResult::Ok(v) => Ok(Ok(v)),
            CallFunctionResult::Err(v) => Ok(Err(v)),
            CallFunctionResult::Aborted => Err(Error::FunctionCallAborted),
            CallFunctionResult::InvalidService => Err(Error::InvalidService(self.service_id)),
            CallFunctionResult::InvalidFunction => {
                Err(Error::InvalidFunction(self.service_id, self.function))
            }
            CallFunctionResult::InvalidArgs => {
                Err(Error::InvalidArgs(self.service_id, self.function))
            }
        })
    }
}
