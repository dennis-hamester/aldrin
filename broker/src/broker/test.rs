use crate::core::channel::{self, Disconnected, Unbounded};
use crate::core::message::{
    CallFunction, CallFunctionReply, CallFunctionResult, ChannelEndClaimed, ChannelEndClosed,
    ClaimChannelEnd, ClaimChannelEndReply, ClaimChannelEndResult, CloseChannelEnd,
    CloseChannelEndReply, CloseChannelEndResult, Connect, Connect2, ConnectData, ConnectReply,
    ConnectResult, CreateChannel, CreateChannelReply, CreateObject, CreateObjectReply,
    CreateObjectResult, CreateService, CreateServiceReply, CreateServiceResult, Message, SendItem,
    Sync, SyncReply,
};
use crate::core::transport::AsyncTransportExt;
use crate::core::{
    ChannelEnd, ChannelEndWithCapacity, ObjectUuid, ProtocolVersion, SerializedValue, ServiceUuid,
};
use crate::{Broker, BrokerHandle};
use aldrin::low_level::{Proxy, ServiceInfo};
use aldrin::Client;
use aldrin_test::tokio::TestBroker;
use futures_util::future::{self, Either};
use std::future::Future;
use std::mem;
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
    let reply = proxy.call(0, &(), None);
    mem::drop(reply);
    client2.join().await;

    let call = svc.next_call().await.unwrap();
    call.into_promise().ok(&()).unwrap();
    client1.join().await;

    broker.join_idle().await
}

#[tokio::test]
async fn drop_conn_before_function_call() {
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

    async fn select_first<F1: Future, F2: Future>(f1: F1, f2: F2) -> F1::Output {
        match future::select(Box::pin(f1), Box::pin(f2)).await {
            Either::Left((res, _)) => res,
            Either::Right(_) => unreachable!(),
        }
    }

    let obj = client1.create_object(ObjectUuid::new_v4());
    let obj = select_first(obj, &mut conn1_fut).await.unwrap();

    let info = ServiceInfo::new(0);
    let svc = obj.create_service(ServiceUuid::new_v4(), info);
    let svc = select_first(svc, &mut conn1_fut).await.unwrap();

    // This will cause all subsequent sends in the broker to fail.
    mem::drop(conn1_fut);

    // Calling a function on conn1 must not deadlock, but be immediately replied to with an error.
    let proxy = Proxy::new(&client2, svc.id()).await.unwrap();
    let res = time::timeout(Duration::from_millis(500), proxy.call(0, &(), None))
        .await
        .unwrap();

    assert!(res.is_err());
}

#[tokio::test]
async fn begin_connect_accept() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    let join = tokio::spawn(broker.run());

    let (mut t1, t2) = channel::unbounded();

    t1.send_and_flush(Connect::with_serialize_value(14, &0u32).unwrap())
        .await
        .unwrap();

    let conn = handle.begin_connect(t2).await.unwrap();
    assert_eq!(conn.deserialize_client_data(), Some(Ok(0u32)));

    let _ = conn.accept_serialize(Some(&1u32)).await.unwrap();
    let value = match t1.receive().await.unwrap() {
        Message::ConnectReply(ConnectReply::Ok(value)) => value,
        msg => panic!("invalid msg received {msg:?}"),
    };
    assert_eq!(value.deserialize(), Ok(1u32));

    handle.shutdown().await;
    join.await.unwrap();
}

#[tokio::test]
async fn begin_connect_2_accept() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    let join = tokio::spawn(broker.run());

    let (mut t1, t2) = channel::unbounded();

    let mut data = ConnectData::new();
    data.serialize_user(&0u32).unwrap();
    t1.send_and_flush(
        Connect2::with_serialize_data(
            ProtocolVersion::V1_14.major(),
            ProtocolVersion::V1_15.minor(),
            &data,
        )
        .unwrap(),
    )
    .await
    .unwrap();

    let conn = handle.begin_connect(t2).await.unwrap();
    assert_eq!(conn.deserialize_client_data(), Some(Ok(0u32)));

    let _ = conn.accept_serialize(Some(&1u32)).await.unwrap();
    let msg = match t1.receive().await.unwrap() {
        Message::ConnectReply2(msg) => msg,
        msg => panic!("invalid msg received {msg:?}"),
    };
    assert_eq!(
        msg.result,
        ConnectResult::Ok(ProtocolVersion::V1_15.minor())
    );
    let data = msg.deserialize_connect_data().unwrap();
    assert_eq!(data.deserialize_user(), Some(Ok(1u32)));

    handle.shutdown().await;
    join.await.unwrap();
}

#[tokio::test]
async fn begin_connect_reject() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    let join = tokio::spawn(broker.run());

    let (mut t1, t2) = channel::unbounded();

    t1.send_and_flush(Connect::with_serialize_value(14, &0u32).unwrap())
        .await
        .unwrap();

    let conn = handle.begin_connect(t2).await.unwrap();
    assert_eq!(conn.deserialize_client_data(), Some(Ok(0u32)));

    conn.reject_serialize(Some(&1u32)).await.unwrap();
    let value = match t1.receive().await.unwrap() {
        Message::ConnectReply(ConnectReply::Rejected(value)) => value,
        msg => panic!("invalid msg received {msg:?}"),
    };
    assert_eq!(value.deserialize(), Ok(1u32));

    handle.shutdown().await;
    join.await.unwrap();
}

#[tokio::test]
async fn begin_connect_2_reject() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    let join = tokio::spawn(broker.run());

    let (mut t1, t2) = channel::unbounded();

    let mut data = ConnectData::new();
    data.serialize_user(&0u32).unwrap();
    t1.send_and_flush(
        Connect2::with_serialize_data(
            ProtocolVersion::V1_14.major(),
            ProtocolVersion::V1_15.minor(),
            &data,
        )
        .unwrap(),
    )
    .await
    .unwrap();

    let conn = handle.begin_connect(t2).await.unwrap();
    assert_eq!(conn.deserialize_client_data(), Some(Ok(0u32)));

    conn.reject_serialize(Some(&1u32)).await.unwrap();
    let msg = match t1.receive().await.unwrap() {
        Message::ConnectReply2(msg) => msg,
        msg => panic!("invalid msg received {msg:?}"),
    };
    assert_eq!(msg.result, ConnectResult::Rejected);
    let data = msg.deserialize_connect_data().unwrap();
    assert_eq!(data.deserialize_user(), Some(Ok(1u32)));

    handle.shutdown().await;
    join.await.unwrap();
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
        .send(Message::CallFunction(
            CallFunction::with_serialize_value(0, service_cookie, 0, &()).unwrap(),
        ))
        .await
        .unwrap();

    let Message::CallFunction(CallFunction { serial, .. }) = client1.receive().await.unwrap()
    else {
        panic!();
    };

    // Here, client2 replies to the function call that client1 received.
    client2
        .send(Message::CallFunctionReply(
            CallFunctionReply::err_with_serialize_value(serial, &()).unwrap(),
        ))
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
        .send(Message::CallFunctionReply(
            CallFunctionReply::ok_with_serialize_value(serial, &()).unwrap(),
        ))
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
    };

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
        .send(Message::SendItem(
            SendItem::with_serialize_value(cookie, &()).unwrap(),
        ))
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
    };

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
    };
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
    let reply = proxy.call(0, &(), None);
    svc.next_call()
        .await
        .unwrap()
        .into_promise()
        .done()
        .unwrap();
    reply.await.unwrap().into_args().unwrap();

    let mut client3 = broker.add_client().await;
    let proxy = client3.create_proxy(svc.id()).await.unwrap();
    let reply = proxy.call(0, &(), None);
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
            value: SerializedValue::serialize(&()).unwrap(),
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
            value: SerializedValue::serialize(&()).unwrap(),
        }))
        .await
        .unwrap();

    assert_eq!(client.receive().await, Err(Disconnected));
}

async fn connect_client(broker: &mut BrokerHandle) -> Unbounded {
    const VERSION: ProtocolVersion = ProtocolVersion::V1_16;

    let (mut t1, t2) = channel::unbounded();

    t1.send(
        Connect2::with_serialize_data(VERSION.major(), VERSION.minor(), &ConnectData::new())
            .unwrap(),
    )
    .await
    .unwrap();

    let conn = broker.connect(t2).await.unwrap();

    let Message::ConnectReply2(reply) = t1.receive().await.unwrap() else {
        panic!("expected connect-reply2");
    };

    assert_eq!(reply.result, ConnectResult::Ok(VERSION.minor()));

    tokio::spawn(conn.run());
    t1
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
    let reply = proxy.call(0, &(), None);

    reply.abort();
    svc.destroy().await.unwrap();

    client1.join().await;
    client2.join().await;
    broker.join().await;
}
