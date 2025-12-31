use super::ClientBuilder;
use crate::error::ConnectError;
use aldrin_broker::{Acceptor, Broker};
use aldrin_core::{ProtocolVersion, channel};

#[tokio::test]
async fn connect_with_data_accept() {
    let (t1, t2) = channel::unbounded();

    tokio::spawn(async {
        let broker = Broker::new();
        let mut handle = broker.handle().clone();
        let mut acceptor = Acceptor::new(t2).await.unwrap();

        assert!(acceptor.version() > ProtocolVersion::V1_14);

        assert_eq!(
            acceptor
                .deserialize_client_data::<String>()
                .unwrap()
                .unwrap(),
            "foo"
        );

        acceptor.serialize_reply_data("bar").unwrap();
        let _ = acceptor.accept(&mut handle).await.unwrap();
    });

    let mut builder = ClientBuilder::new(t1);
    builder.serialize_data("foo").unwrap();

    let (_, data) = builder.connect_with_data().await.unwrap();
    assert_eq!(data.unwrap().deserialize::<String>().unwrap(), "bar");
}

#[tokio::test]
async fn connect_with_data_reject() {
    let (t1, t2) = channel::unbounded();

    tokio::spawn(async {
        let mut acceptor = Acceptor::new(t2).await.unwrap();

        assert!(acceptor.version() > ProtocolVersion::V1_14);

        assert_eq!(
            acceptor
                .deserialize_client_data::<String>()
                .unwrap()
                .unwrap(),
            "foo"
        );

        acceptor.serialize_reply_data("bar").unwrap();
        acceptor.reject().await.unwrap();
    });

    let mut builder = ClientBuilder::new(t1);
    builder.serialize_data("foo").unwrap();

    let data = match builder.connect_with_data().await.unwrap_err() {
        ConnectError::Rejected(Some(data)) => data,
        _ => panic!(),
    };

    assert_eq!(data.deserialize::<String>().unwrap(), "bar");
}

#[tokio::test]
async fn connect_accept() {
    let (t1, t2) = channel::unbounded();

    tokio::spawn(async {
        let broker = Broker::new();
        let mut handle = broker.handle().clone();
        let acceptor = Acceptor::new(t2).await.unwrap();

        assert!(acceptor.version() > ProtocolVersion::V1_14);
        assert_eq!(acceptor.client_data(), None);

        let _ = acceptor.accept(&mut handle).await.unwrap();
    });

    let builder = ClientBuilder::new(t1);
    let _ = builder.connect().await.unwrap();
}

#[tokio::test]
async fn connect_reject() {
    let (t1, t2) = channel::unbounded();

    tokio::spawn(async {
        let acceptor = Acceptor::new(t2).await.unwrap();

        assert!(acceptor.version() > ProtocolVersion::V1_14);
        assert_eq!(acceptor.client_data(), None);

        acceptor.reject().await.unwrap();
    });

    let builder = ClientBuilder::new(t1);

    let data = match builder.connect().await.unwrap_err() {
        ConnectError::Rejected(data) => data,
        _ => panic!(),
    };

    assert_eq!(data, None);
}

#[tokio::test]
async fn connect1_with_data_accept() {
    let (t1, t2) = channel::unbounded();

    tokio::spawn(async {
        let broker = Broker::new();
        let mut handle = broker.handle().clone();
        let mut acceptor = Acceptor::new(t2).await.unwrap();

        assert!(acceptor.version() == ProtocolVersion::V1_14);

        assert_eq!(
            acceptor
                .deserialize_client_data::<String>()
                .unwrap()
                .unwrap(),
            "foo"
        );

        acceptor.serialize_reply_data("bar").unwrap();
        let _ = acceptor.accept(&mut handle).await.unwrap();
    });

    let mut builder = ClientBuilder::new(t1);
    builder.serialize_data("foo").unwrap();

    let (_, data) = builder.connect1_with_data().await.unwrap();
    assert_eq!(data.deserialize::<String>().unwrap(), "bar");
}

#[tokio::test]
async fn connect1_with_data_reject() {
    let (t1, t2) = channel::unbounded();

    tokio::spawn(async {
        let mut acceptor = Acceptor::new(t2).await.unwrap();

        assert!(acceptor.version() == ProtocolVersion::V1_14);

        assert_eq!(
            acceptor
                .deserialize_client_data::<String>()
                .unwrap()
                .unwrap(),
            "foo"
        );

        acceptor.serialize_reply_data("bar").unwrap();
        acceptor.reject().await.unwrap();
    });

    let mut builder = ClientBuilder::new(t1);
    builder.serialize_data("foo").unwrap();

    let data = match builder.connect1_with_data().await.unwrap_err() {
        ConnectError::Rejected(Some(data)) => data,
        _ => panic!(),
    };

    assert_eq!(data.deserialize::<String>().unwrap(), "bar");
}

#[tokio::test]
async fn connect1_accept() {
    let (t1, t2) = channel::unbounded();

    tokio::spawn(async {
        let broker = Broker::new();
        let mut handle = broker.handle().clone();
        let acceptor = Acceptor::new(t2).await.unwrap();

        assert!(acceptor.version() == ProtocolVersion::V1_14);
        acceptor.client_data().unwrap().deserialize::<()>().unwrap();

        let _ = acceptor.accept(&mut handle).await.unwrap();
    });

    let builder = ClientBuilder::new(t1);
    let _ = builder.connect1().await.unwrap();
}

#[tokio::test]
async fn connect1_reject() {
    let (t1, t2) = channel::unbounded();

    tokio::spawn(async {
        let acceptor = Acceptor::new(t2).await.unwrap();

        assert!(acceptor.version() == ProtocolVersion::V1_14);
        acceptor.client_data().unwrap().deserialize::<()>().unwrap();

        acceptor.reject().await.unwrap();
    });

    let builder = ClientBuilder::new(t1);

    let data = match builder.connect1().await.unwrap_err() {
        ConnectError::Rejected(Some(data)) => data,
        _ => panic!(),
    };

    data.deserialize::<()>().unwrap();
}
