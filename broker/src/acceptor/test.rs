use super::{Acceptor, select_protocol_version};
use crate::Broker;
use aldrin_core::message::{
    Connect, Connect2, ConnectData, ConnectReply, ConnectReplyData, ConnectResult, Message,
    MessageOps,
};
use aldrin_core::transport::AsyncTransportExt;
use aldrin_core::{ProtocolVersion, SerializedValue, channel};

#[test]
fn select_protocol_version_connect1() {
    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 14), false),
        Some(ProtocolVersion::V1_14)
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 15), false),
        None
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 13), false),
        None
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(2, 0), false),
        None
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(2, 14), false),
        None
    );
}

#[test]
fn select_protocol_version_connect2() {
    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 14), true),
        Some(ProtocolVersion::V1_14)
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 15), true),
        Some(ProtocolVersion::V1_15)
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 16), true),
        Some(ProtocolVersion::V1_16)
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 17), true),
        Some(ProtocolVersion::V1_17)
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 18), true),
        Some(ProtocolVersion::V1_18)
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 19), true),
        Some(ProtocolVersion::V1_19)
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 20), true),
        Some(ProtocolVersion::V1_20)
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 21), true),
        Some(ProtocolVersion::V1_20)
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(1, 13), true),
        None
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(2, 0), true),
        None
    );

    assert_eq!(
        select_protocol_version(ProtocolVersion::new(2, 14), true),
        None
    );
}

#[tokio::test]
async fn connect1_accept() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    let join = tokio::spawn(broker.run());

    let (mut t1, t2) = channel::unbounded();

    t1.send_and_flush(Connect {
        version: 14,
        value: SerializedValue::serialize(0u32).unwrap(),
    })
    .await
    .unwrap();

    let mut acceptor = Acceptor::new(t2).await.unwrap();
    assert_eq!(acceptor.deserialize_client_data(), Some(Ok(0u32)));

    acceptor.serialize_reply_data(1u32).unwrap();
    let _ = acceptor.accept(&mut handle).await.unwrap();

    #[expect(clippy::wildcard_enum_match_arm)]
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
    data.serialize_user(0u32).unwrap();

    let mut msg = Connect2 {
        major_version: ProtocolVersion::V1_15.major(),
        minor_version: ProtocolVersion::V1_15.minor(),
        value: SerializedValue::serialize(data).unwrap(),
    };

    msg.convert_value(None, ProtocolVersion::V1_15).unwrap();
    t1.send_and_flush(msg).await.unwrap();

    let mut acceptor = Acceptor::new(t2).await.unwrap();
    assert_eq!(acceptor.deserialize_client_data(), Some(Ok(0u32)));

    acceptor.serialize_reply_data(1u32).unwrap();
    let _ = acceptor.accept(&mut handle).await.unwrap();

    #[expect(clippy::wildcard_enum_match_arm)]
    let msg = match t1.receive().await.unwrap() {
        Message::ConnectReply2(msg) => msg,
        msg => panic!("invalid msg received {msg:?}"),
    };
    assert_eq!(
        msg.result,
        ConnectResult::Ok(ProtocolVersion::V1_15.minor())
    );

    let data = msg.value.deserialize::<ConnectReplyData>().unwrap();
    assert_eq!(data.deserialize_user(), Some(Ok(1u32)));

    handle.shutdown().await;
    join.await.unwrap();
}

#[tokio::test]
async fn connect1_reject() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    let join = tokio::spawn(broker.run());

    let (mut t1, t2) = channel::unbounded();

    t1.send_and_flush(Connect {
        version: 14,
        value: SerializedValue::serialize(0u32).unwrap(),
    })
    .await
    .unwrap();

    let mut acceptor = Acceptor::new(t2).await.unwrap();
    assert_eq!(acceptor.deserialize_client_data(), Some(Ok(0u32)));

    acceptor.serialize_reply_data(1u32).unwrap();
    acceptor.reject().await.unwrap();

    #[expect(clippy::wildcard_enum_match_arm)]
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
    data.serialize_user(0u32).unwrap();

    let mut msg = Connect2 {
        major_version: ProtocolVersion::V1_15.major(),
        minor_version: ProtocolVersion::V1_15.minor(),
        value: SerializedValue::serialize(data).unwrap(),
    };

    msg.convert_value(None, ProtocolVersion::V1_15).unwrap();
    t1.send_and_flush(msg).await.unwrap();

    let mut acceptor = Acceptor::new(t2).await.unwrap();
    assert_eq!(acceptor.deserialize_client_data(), Some(Ok(0u32)));

    acceptor.serialize_reply_data(1u32).unwrap();
    acceptor.reject().await.unwrap();

    #[expect(clippy::wildcard_enum_match_arm)]
    let msg = match t1.receive().await.unwrap() {
        Message::ConnectReply2(msg) => msg,
        msg => panic!("invalid msg received {msg:?}"),
    };
    assert_eq!(msg.result, ConnectResult::Rejected);

    let data = msg.value.deserialize::<ConnectReplyData>().unwrap();
    assert_eq!(data.deserialize_user(), Some(Ok(1u32)));

    handle.shutdown().await;
    join.await.unwrap();
}
