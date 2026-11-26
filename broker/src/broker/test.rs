use crate::{Broker, BrokerHandle};
use aldrin::Client;
use aldrin::low_level::{Proxy, ServiceInfo};
#[cfg(feature = "introspection")]
use aldrin_core::TypeId;
use aldrin_core::channel::{self, Disconnected, Unbounded};
use aldrin_core::message::{
    CallFunction, CallFunction2, CallFunctionReply, CallFunctionResult, ChannelEndClaimed,
    ChannelEndClosed, ClaimChannelEnd, ClaimChannelEndReply, ClaimChannelEndResult,
    CloseChannelEnd, CloseChannelEndReply, CloseChannelEndResult, Connect2, ConnectData,
    ConnectResult, CreateChannel, CreateChannelReply, CreateObject, CreateObjectReply,
    CreateObjectResult, CreateService, CreateServiceReply, CreateServiceResult, EmitEvent, Message,
    MessageOps, SendItem, ServiceDestroyed, SubscribeEvent, SubscribeEventReply,
    SubscribeEventResult, Sync, SyncReply,
};
#[cfg(feature = "introspection")]
use aldrin_core::message::{
    QueryIntrospection, QueryIntrospectionReply, QueryIntrospectionResult, RegisterIntrospection,
};
use aldrin_core::transport::AsyncTransportExt;
use aldrin_core::{
    ChannelEnd, ChannelEndWithCapacity, ObjectUuid, ProtocolVersion, SerializedValue,
    ServiceCookie, ServiceUuid,
};
use aldrin_test::tokio::TestBroker;
use futures_util::future::{self, Either};
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn disconnect_during_function_call() {
    let mut broker = TestBroker::new();

    let mut client1 = broker.add_client().await;
    let obj = client1.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    // client2 calls a function on client1 and disconnects before client1 replies.
    let mut client2 = broker.add_client().await;
    let proxy = Proxy::new(&client2, svc.id()).await.unwrap();
    let reply = proxy.call(0, (), None);
    drop(reply);
    client2.join().await;

    let call = svc.next_call().await.unwrap();
    call.into_promise().ok(()).unwrap();
    client1.join().await;

    broker.join_idle().await;
}

#[tokio::test]
async fn drop_conn_before_function_call() {
    async fn select_first<F1: Future, F2: Future>(f1: F1, f2: F2) -> F1::Output {
        match future::select(Box::pin(f1), Box::pin(f2)).await {
            Either::Left((res, _)) => res,
            Either::Right(_) => unreachable!(),
        }
    }

    let mut broker = TestBroker::new();

    // Setup client1 manually, such that we can drop its connection future (conn1_fut) at the right
    // moment.
    let (t1, t2) = channel::unbounded();
    let client1_fut = Client::connect(t1);
    let conn1_fut = broker.connect(t2);
    let (client1_fut, conn1_fut) = future::join(client1_fut, conn1_fut).await;
    let client1_fut = client1_fut.unwrap();
    let client1 = client1_fut.handle().clone();
    tokio::spawn(client1_fut.run());
    let mut conn1_fut = Box::pin(conn1_fut.unwrap().run());

    let client2 = broker.add_client().await;

    let obj = client1.create_object(ObjectUuid::new_v4());
    let obj = select_first(obj, &mut conn1_fut).await.unwrap();

    let info = ServiceInfo::new(0);
    let svc = obj.create_service(ServiceUuid::new_v4(), info);
    let svc = select_first(svc, &mut conn1_fut).await.unwrap();

    // This will cause all subsequent sends in the broker to fail.
    drop(conn1_fut);

    // Calling a function on conn1 must not deadlock, but be immediately replied to with an error.
    let proxy = Proxy::new(&client2, svc.id()).await.unwrap();
    let res = time::timeout(Duration::from_millis(500), proxy.call(0, (), None))
        .await
        .unwrap();

    assert!(res.is_err());
}

#[tokio::test]
async fn wrong_client_replies_function_call() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    tokio::spawn(broker.run());

    let mut client1 = connect_client(&mut handle).await;
    let mut client2 = connect_client(&mut handle).await;

    let object_uuid = ObjectUuid::new_v4();

    client1
        .send(Message::CreateObject(CreateObject {
            serial: 0,
            uuid: object_uuid,
        }))
        .await
        .unwrap();

    let Message::CreateObjectReply(CreateObjectReply {
        result: CreateObjectResult::Ok(object_cookie),
        ..
    }) = client1.receive().await.unwrap()
    else {
        panic!();
    };

    let service_uuid = ServiceUuid::new_v4();

    client1
        .send(Message::CreateService(CreateService {
            serial: 0,
            object_cookie,
            uuid: service_uuid,
            version: 0,
        }))
        .await
        .unwrap();

    let Message::CreateServiceReply(CreateServiceReply {
        result: CreateServiceResult::Ok(service_cookie),
        ..
    }) = client1.receive().await.unwrap()
    else {
        panic!();
    };

    client1
        .send(Message::CallFunction(CallFunction {
            serial: 0,
            service_cookie,
            function: 0,
            value: SerializedValue::serialize(()).unwrap(),
        }))
        .await
        .unwrap();

    let Message::CallFunction(CallFunction { serial, .. }) = client1.receive().await.unwrap()
    else {
        panic!();
    };

    // Here, client2 replies to the function call that client1 received.
    client2
        .send(Message::CallFunctionReply(CallFunctionReply {
            serial,
            result: CallFunctionResult::Err(SerializedValue::serialize(()).unwrap()),
        }))
        .await
        .unwrap();

    // Sync client2 to make sure the CallFunctionReply has been processed by the broker.
    client2
        .send(Message::Sync(Sync { serial: 0 }))
        .await
        .unwrap();

    let Message::SyncReply(SyncReply { .. }) = client2.receive().await.unwrap() else {
        panic!();
    };

    client1
        .send(Message::CallFunctionReply(CallFunctionReply {
            serial,
            result: CallFunctionResult::Ok(SerializedValue::serialize(()).unwrap()),
        }))
        .await
        .unwrap();

    let Message::CallFunctionReply(CallFunctionReply {
        result: CallFunctionResult::Ok(_),
        ..
    }) = client1.receive().await.unwrap()
    else {
        panic!();
    };
}

#[tokio::test]
async fn send_item_without_capacity() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    tokio::spawn(broker.run());

    let mut client1 = connect_client(&mut handle).await;
    let mut client2 = connect_client(&mut handle).await;

    client1
        .send(Message::CreateChannel(CreateChannel {
            serial: 0,
            end: ChannelEndWithCapacity::Sender,
        }))
        .await
        .unwrap();

    let Message::CreateChannelReply(CreateChannelReply { serial: 0, cookie }) =
        client1.receive().await.unwrap()
    else {
        panic!();
    };

    client2
        .send(Message::ClaimChannelEnd(ClaimChannelEnd {
            serial: 0,
            cookie,
            end: ChannelEndWithCapacity::Receiver(0),
        }))
        .await
        .unwrap();

    if !matches!(
        client2.receive().await,
        Ok(Message::ClaimChannelEndReply(ClaimChannelEndReply {
            serial: 0,
            result: ClaimChannelEndResult::ReceiverClaimed,
        })),
    ) {
        panic!();
    }

    if !matches!(
        client1.receive().await,
        Ok(Message::ChannelEndClaimed(ChannelEndClaimed {
            cookie: cookie2,
            end: ChannelEndWithCapacity::Receiver(0),
        }))
        if cookie == cookie2
    ) {
        panic!()
    }

    client1
        .send(Message::SendItem(SendItem {
            cookie,
            value: SerializedValue::serialize(()).unwrap(),
        }))
        .await
        .unwrap();

    if !matches!(
        client2.receive().await,
        Ok(Message::ChannelEndClosed(ChannelEndClosed {
            cookie: cookie2,
            end: ChannelEnd::Sender,
        }))
        if cookie == cookie2
    ) {
        panic!();
    }

    client2
        .send(Message::CloseChannelEnd(CloseChannelEnd {
            serial: 0,
            cookie,
            end: ChannelEnd::Receiver,
        }))
        .await
        .unwrap();

    if !matches!(
        client2.receive().await,
        Ok(Message::CloseChannelEndReply(CloseChannelEndReply {
            serial: 0,
            result: CloseChannelEndResult::Ok,
        }))
    ) {
        panic!();
    }
}

#[tokio::test]
async fn calls_from_multiple_clients() {
    let mut broker = TestBroker::new();

    let mut client1 = broker.add_client().await;
    let obj = client1.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let mut client2 = broker.add_client().await;
    let proxy = client2.create_proxy(svc.id()).await.unwrap();
    let reply = proxy.call(0, (), None);
    svc.next_call()
        .await
        .unwrap()
        .into_promise()
        .done()
        .unwrap();
    reply.await.unwrap().into_args().unwrap();

    let mut client3 = broker.add_client().await;
    let proxy = client3.create_proxy(svc.id()).await.unwrap();
    let reply = proxy.call(0, (), None);
    svc.next_call()
        .await
        .unwrap()
        .into_promise()
        .done()
        .unwrap();
    reply.await.unwrap().into_args().unwrap();

    client1.join().await;
    client2.join().await;
    client3.join().await;
    broker.join().await;
}

#[tokio::test]
async fn duplicate_call_serial() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    tokio::spawn(broker.run());

    let mut client = connect_client(&mut handle).await;

    let object_uuid = ObjectUuid::new_v4();

    client
        .send(Message::CreateObject(CreateObject {
            serial: 0,
            uuid: object_uuid,
        }))
        .await
        .unwrap();

    let Message::CreateObjectReply(CreateObjectReply {
        result: CreateObjectResult::Ok(object_cookie),
        ..
    }) = client.receive().await.unwrap()
    else {
        panic!();
    };

    let service_uuid = ServiceUuid::new_v4();

    client
        .send(Message::CreateService(CreateService {
            serial: 0,
            object_cookie,
            uuid: service_uuid,
            version: 0,
        }))
        .await
        .unwrap();

    let Message::CreateServiceReply(CreateServiceReply {
        result: CreateServiceResult::Ok(service_cookie),
        ..
    }) = client.receive().await.unwrap()
    else {
        panic!();
    };

    client
        .send(Message::CallFunction(CallFunction {
            serial: 0,
            service_cookie,
            function: 0,
            value: SerializedValue::serialize(()).unwrap(),
        }))
        .await
        .unwrap();

    assert!(matches!(
        client.receive().await.unwrap(),
        Message::CallFunction(_)
    ));

    client
        .send(Message::CallFunction(CallFunction {
            serial: 0,
            service_cookie,
            function: 0,
            value: SerializedValue::serialize(()).unwrap(),
        }))
        .await
        .unwrap();

    assert_eq!(client.receive().await, Err(Disconnected));
}

async fn connect_client(broker: &mut BrokerHandle) -> Unbounded {
    connect_client_version(broker, ProtocolVersion::V1_16).await
}

async fn connect_client_version(broker: &mut BrokerHandle, version: ProtocolVersion) -> Unbounded {
    let (mut t1, t2) = channel::unbounded();

    t1.send(Connect2 {
        major_version: version.major(),
        minor_version: version.minor(),
        value: SerializedValue::serialize(ConnectData::new()).unwrap(),
    })
    .await
    .unwrap();

    let conn = broker.connect(t2).await.unwrap();

    let Message::ConnectReply2(reply) = t1.receive().await.unwrap() else {
        panic!("expected connect-reply2");
    };

    assert_eq!(reply.result, ConnectResult::Ok(version.minor()));

    tokio::spawn(conn.run());
    t1
}

fn malformed_value() -> SerializedValue {
    let mut buf = CallFunction {
        serial: 0,
        service_cookie: ServiceCookie::new_v4(),
        function: 0,
        value: SerializedValue::empty(),
    }
    .serialize_message()
    .unwrap();
    buf[9] = 255;

    let Message::CallFunction(msg) = Message::deserialize_message(buf).unwrap() else {
        unreachable!();
    };

    msg.value
}

async fn create_object_and_service(client: &mut Unbounded) -> ServiceCookie {
    client
        .send(Message::CreateObject(CreateObject {
            serial: 0,
            uuid: ObjectUuid::new_v4(),
        }))
        .await
        .unwrap();

    let Message::CreateObjectReply(CreateObjectReply {
        result: CreateObjectResult::Ok(object_cookie),
        ..
    }) = client.receive().await.unwrap()
    else {
        panic!();
    };

    client
        .send(Message::CreateService(CreateService {
            serial: 0,
            object_cookie,
            uuid: ServiceUuid::new_v4(),
            version: 0,
        }))
        .await
        .unwrap();

    let Message::CreateServiceReply(CreateServiceReply {
        result: CreateServiceResult::Ok(service_cookie),
        ..
    }) = client.receive().await.unwrap()
    else {
        panic!();
    };

    service_cookie
}

#[tokio::test]
async fn destroy_service_after_abort() {
    let mut broker = TestBroker::new();

    let mut client1 = broker.add_client().await;
    let obj = client1.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let mut client2 = broker.add_client().await;
    let proxy = client2.create_proxy(svc.id()).await.unwrap();
    let reply = proxy.call(0, (), None);

    reply.abort();
    svc.destroy().await.unwrap();

    client1.join().await;
    client2.join().await;
    broker.join().await;
}

#[tokio::test]
async fn malformed_call_function_value_does_not_disconnect_callee() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    tokio::spawn(broker.run());

    let mut callee = connect_client_version(&mut handle, ProtocolVersion::V1_19).await;
    let mut caller = connect_client_version(&mut handle, ProtocolVersion::V1_20).await;
    let mut caller2 = connect_client_version(&mut handle, ProtocolVersion::V1_20).await;

    let service_cookie = create_object_and_service(&mut callee).await;

    caller
        .send(Message::CallFunction(CallFunction {
            serial: 0,
            service_cookie,
            function: 0,
            value: malformed_value(),
        }))
        .await
        .unwrap();

    assert_eq!(caller.receive().await, Err(Disconnected));

    caller2
        .send(Message::CallFunction2(CallFunction2 {
            serial: 0,
            service_cookie,
            function: 0,
            version: None,
            value: malformed_value(),
        }))
        .await
        .unwrap();

    assert_eq!(caller2.receive().await, Err(Disconnected));

    callee
        .send(Message::Sync(Sync { serial: 1 }))
        .await
        .unwrap();
    assert_eq!(
        callee.receive().await,
        Ok(Message::SyncReply(SyncReply { serial: 1 }))
    );
}

#[tokio::test]
async fn malformed_call_function_reply_value_does_not_disconnect_caller() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    tokio::spawn(broker.run());

    let mut callee = connect_client_version(&mut handle, ProtocolVersion::V1_20).await;
    let mut caller = connect_client_version(&mut handle, ProtocolVersion::V1_19).await;

    let service_cookie = create_object_and_service(&mut callee).await;

    caller
        .send(Message::CallFunction(CallFunction {
            serial: 0,
            service_cookie,
            function: 0,
            value: SerializedValue::empty(),
        }))
        .await
        .unwrap();

    let Message::CallFunction2(CallFunction2 { serial, .. }) = callee.receive().await.unwrap()
    else {
        panic!();
    };

    callee
        .send(Message::CallFunctionReply(CallFunctionReply {
            serial,
            result: CallFunctionResult::Ok(malformed_value()),
        }))
        .await
        .unwrap();

    assert_eq!(callee.receive().await, Err(Disconnected));

    assert_eq!(
        caller.receive().await,
        Ok(Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::InvalidService,
        }))
    );

    caller
        .send(Message::Sync(Sync { serial: 1 }))
        .await
        .unwrap();
    assert_eq!(
        caller.receive().await,
        Ok(Message::SyncReply(SyncReply { serial: 1 }))
    );
}

#[tokio::test]
async fn malformed_emit_event_value_does_not_disconnect_subscriber() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    tokio::spawn(broker.run());

    let mut emitter = connect_client_version(&mut handle, ProtocolVersion::V1_20).await;
    let mut subscriber = connect_client_version(&mut handle, ProtocolVersion::V1_19).await;

    let service_cookie = create_object_and_service(&mut emitter).await;

    subscriber
        .send(Message::SubscribeEvent(SubscribeEvent {
            serial: Some(0),
            service_cookie,
            event: 0,
        }))
        .await
        .unwrap();

    assert_eq!(
        subscriber.receive().await,
        Ok(Message::SubscribeEventReply(SubscribeEventReply {
            serial: 0,
            result: SubscribeEventResult::Ok,
        }))
    );

    let Message::SubscribeEvent(_) = emitter.receive().await.unwrap() else {
        panic!();
    };

    emitter
        .send(Message::EmitEvent(EmitEvent {
            service_cookie,
            event: 0,
            value: malformed_value(),
        }))
        .await
        .unwrap();

    assert_eq!(emitter.receive().await, Err(Disconnected));

    assert_eq!(
        subscriber.receive().await,
        Ok(Message::ServiceDestroyed(ServiceDestroyed {
            service_cookie,
        }))
    );

    subscriber
        .send(Message::Sync(Sync { serial: 1 }))
        .await
        .unwrap();
    assert_eq!(
        subscriber.receive().await,
        Ok(Message::SyncReply(SyncReply { serial: 1 }))
    );
}

#[tokio::test]
async fn malformed_send_item_value_does_not_disconnect_receiver() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    tokio::spawn(broker.run());

    let mut sender = connect_client_version(&mut handle, ProtocolVersion::V1_20).await;
    let mut receiver = connect_client_version(&mut handle, ProtocolVersion::V1_19).await;

    sender
        .send(Message::CreateChannel(CreateChannel {
            serial: 0,
            end: ChannelEndWithCapacity::Sender,
        }))
        .await
        .unwrap();

    let Message::CreateChannelReply(CreateChannelReply { serial: 0, cookie }) =
        sender.receive().await.unwrap()
    else {
        panic!();
    };

    receiver
        .send(Message::ClaimChannelEnd(ClaimChannelEnd {
            serial: 0,
            cookie,
            end: ChannelEndWithCapacity::Receiver(1),
        }))
        .await
        .unwrap();

    assert_eq!(
        receiver.receive().await,
        Ok(Message::ClaimChannelEndReply(ClaimChannelEndReply {
            serial: 0,
            result: ClaimChannelEndResult::ReceiverClaimed,
        }))
    );

    assert_eq!(
        sender.receive().await,
        Ok(Message::ChannelEndClaimed(ChannelEndClaimed {
            cookie,
            end: ChannelEndWithCapacity::Receiver(1),
        }))
    );

    sender
        .send(Message::SendItem(SendItem {
            cookie,
            value: malformed_value(),
        }))
        .await
        .unwrap();

    assert_eq!(sender.receive().await, Err(Disconnected));

    assert_eq!(
        receiver.receive().await,
        Ok(Message::ChannelEndClosed(ChannelEndClosed {
            cookie,
            end: ChannelEnd::Sender,
        }))
    );

    receiver
        .send(Message::Sync(Sync { serial: 1 }))
        .await
        .unwrap();
    assert_eq!(
        receiver.receive().await,
        Ok(Message::SyncReply(SyncReply { serial: 1 }))
    );
}

#[cfg(feature = "introspection")]
#[tokio::test]
async fn malformed_query_introspection_reply_does_not_disconnect_querier() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    tokio::spawn(broker.run());

    let mut provider = connect_client_version(&mut handle, ProtocolVersion::V1_20).await;
    let mut querier = connect_client_version(&mut handle, ProtocolVersion::V1_19).await;

    let type_id = TypeId::new_v4();

    provider
        .send(Message::RegisterIntrospection(RegisterIntrospection {
            value: SerializedValue::serialize(std::collections::HashSet::from([type_id])).unwrap(),
        }))
        .await
        .unwrap();

    provider
        .send(Message::Sync(Sync { serial: 0 }))
        .await
        .unwrap();
    assert_eq!(
        provider.receive().await,
        Ok(Message::SyncReply(SyncReply { serial: 0 }))
    );

    querier
        .send(Message::QueryIntrospection(QueryIntrospection {
            serial: 0,
            type_id,
        }))
        .await
        .unwrap();

    let Message::QueryIntrospection(QueryIntrospection { serial, .. }) =
        provider.receive().await.unwrap()
    else {
        panic!();
    };

    provider
        .send(Message::QueryIntrospectionReply(QueryIntrospectionReply {
            serial,
            result: QueryIntrospectionResult::Ok(malformed_value()),
        }))
        .await
        .unwrap();

    assert_eq!(provider.receive().await, Err(Disconnected));

    assert_eq!(
        querier.receive().await,
        Ok(Message::QueryIntrospectionReply(QueryIntrospectionReply {
            serial: 0,
            result: QueryIntrospectionResult::Unavailable,
        }))
    );

    // Ensure the introspection cache was not poisoned.
    querier
        .send(Message::QueryIntrospection(QueryIntrospection {
            serial: 1,
            type_id,
        }))
        .await
        .unwrap();

    assert_eq!(
        querier.receive().await,
        Ok(Message::QueryIntrospectionReply(QueryIntrospectionReply {
            serial: 1,
            result: QueryIntrospectionResult::Unavailable,
        }))
    );

    querier
        .send(Message::Sync(Sync { serial: 2 }))
        .await
        .unwrap();
    assert_eq!(
        querier.receive().await,
        Ok(Message::SyncReply(SyncReply { serial: 2 }))
    );
}
