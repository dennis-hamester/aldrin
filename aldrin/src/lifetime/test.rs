use aldrin_test::aldrin::Error;
use aldrin_test::tokio::TestBroker;

#[tokio::test]
async fn end_explicitly() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let scope = client.create_lifetime_scope().await.unwrap();

    let mut lifetime = client.create_lifetime(scope.id()).await.unwrap();
    assert!(!lifetime.has_ended());

    scope.end().await.unwrap();
    lifetime.ended().await;
    assert!(lifetime.has_ended());

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn end_implicitly() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let scope = client.create_lifetime_scope().await.unwrap();

    let mut lifetime = client.create_lifetime(scope.id()).await.unwrap();
    assert!(!lifetime.has_ended());

    tokio::spawn(async move {
        let _scope = scope;
    });
    lifetime.ended().await;
    assert!(lifetime.has_ended());

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn end_twice() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let scope = client.create_lifetime_scope().await.unwrap();
    assert_eq!(scope.end().await, Ok(()));
    assert_eq!(scope.end().await, Err(Error::InvalidLifetime));

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn end_before_create_lifetime() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let id = client.create_lifetime_scope().await.unwrap().id();

    let mut lifetime = client.create_lifetime(id).await.unwrap();
    assert!(!lifetime.has_ended());

    lifetime.ended().await;
    assert!(lifetime.has_ended());

    client.join().await;
    broker.join().await;
}
