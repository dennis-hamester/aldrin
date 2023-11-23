use aldrin_core::{ObjectUuid, ServiceUuid};
use aldrin_test::aldrin::DiscovererEventKind;
use aldrin_test::tokio::TestBroker;

#[tokio::test]
async fn any_object_no_services() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let mut discoverer = client
        .create_discoverer()
        .add_object(0, None, None)
        .build()
        .await
        .unwrap();

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Created);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    obj.destroy().await.unwrap();
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Destroyed);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    discoverer.stop().await.unwrap();
    assert!(discoverer.next_event().await.is_none());

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn specific_object_no_services() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let _obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();

    let mut discoverer = client
        .create_discoverer()
        .add_object(0, Some(obj.id().uuid), None)
        .build()
        .await
        .unwrap();

    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Created);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    obj.destroy().await.unwrap();
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Destroyed);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    discoverer.stop().await.unwrap();
    assert!(discoverer.next_event().await.is_none());

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn specific_object() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc1 = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();
    let svc2 = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    let mut discoverer = client
        .create_discoverer()
        .add_object(0, Some(obj.id().uuid), [svc1.id().uuid, svc2.id().uuid])
        .build()
        .await
        .unwrap();

    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Created);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());
    assert_eq!(ev.service_id(svc1.id().uuid), svc1.id());
    assert_eq!(ev.service_id(svc2.id().uuid), svc2.id());

    obj.destroy().await.unwrap();
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Destroyed);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    discoverer.stop().await.unwrap();
    assert!(discoverer.next_event().await.is_none());

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn any_object() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc1 = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();
    let svc2 = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    let mut discoverer = client
        .create_discoverer()
        .add_object(0, None, [svc1.id().uuid, svc2.id().uuid])
        .build()
        .await
        .unwrap();

    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Created);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());
    assert_eq!(ev.service_id(svc1.id().uuid), svc1.id());
    assert_eq!(ev.service_id(svc2.id().uuid), svc2.id());

    obj.destroy().await.unwrap();
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Destroyed);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc1 = obj.create_service(svc1.id().uuid, 0).await.unwrap();
    let svc2 = obj.create_service(svc2.id().uuid, 0).await.unwrap();

    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Created);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());
    assert_eq!(ev.service_id(svc1.id().uuid), svc1.id());
    assert_eq!(ev.service_id(svc2.id().uuid), svc2.id());

    obj.destroy().await.unwrap();
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Destroyed);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    discoverer.stop().await.unwrap();
    assert!(discoverer.next_event().await.is_none());

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn empty() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let mut discoverer = client.create_discoverer::<()>().build().await.unwrap();
    assert!(discoverer.next_event().await.is_none());

    client.join().await;
    broker.join().await;
}
