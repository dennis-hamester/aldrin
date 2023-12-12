use crate::core::{ObjectUuid, ServiceUuid};
use aldrin_test::aldrin::DiscovererEventKind;
use aldrin_test::tokio::TestBroker;

#[tokio::test]
async fn any_object_no_services() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let mut discoverer = client
        .create_discoverer()
        .any(0, None)
        .build()
        .await
        .unwrap();

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();

    assert!(!discoverer.finished());
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Created);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    obj.destroy().await.unwrap();
    assert!(!discoverer.finished());
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Destroyed);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    discoverer.stop().await.unwrap();
    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.finished());

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
        .specific(0, obj.id().uuid, None)
        .build()
        .await
        .unwrap();

    assert!(!discoverer.finished());
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Created);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    obj.destroy().await.unwrap();
    assert!(!discoverer.finished());
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Destroyed);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    discoverer.stop().await.unwrap();
    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.finished());

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
        .specific(0, obj.id().uuid, [svc1.id().uuid, svc2.id().uuid])
        .build()
        .await
        .unwrap();

    assert!(!discoverer.finished());
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Created);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());
    assert_eq!(ev.service_id(svc1.id().uuid), svc1.id());
    assert_eq!(ev.service_id(svc2.id().uuid), svc2.id());

    obj.destroy().await.unwrap();
    assert!(!discoverer.finished());
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Destroyed);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    discoverer.stop().await.unwrap();
    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.finished());

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
        .any(0, [svc1.id().uuid, svc2.id().uuid])
        .build()
        .await
        .unwrap();

    assert!(!discoverer.finished());
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Created);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());
    assert_eq!(ev.service_id(svc1.id().uuid), svc1.id());
    assert_eq!(ev.service_id(svc2.id().uuid), svc2.id());

    obj.destroy().await.unwrap();
    assert!(!discoverer.finished());
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Destroyed);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc1 = obj.create_service(svc1.id().uuid, 0).await.unwrap();
    let svc2 = obj.create_service(svc2.id().uuid, 0).await.unwrap();

    assert!(!discoverer.finished());
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Created);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());
    assert_eq!(ev.service_id(svc1.id().uuid), svc1.id());
    assert_eq!(ev.service_id(svc2.id().uuid), svc2.id());

    obj.destroy().await.unwrap();
    assert!(!discoverer.finished());
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Destroyed);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());

    discoverer.stop().await.unwrap();
    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.finished());

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn empty() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let mut discoverer = client.create_discoverer::<()>().build().await.unwrap();
    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.finished());

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn current_only() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc1 = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();
    let svc2 = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    let mut discoverer = client
        .create_discoverer()
        .any(0, [svc1.id().uuid, svc2.id().uuid])
        .build_current_only()
        .await
        .unwrap();

    let _obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let _svc1 = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();
    let _svc2 = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    assert!(!discoverer.finished());
    let ev = discoverer.next_event().await.unwrap();
    assert_eq!(ev.kind(), DiscovererEventKind::Created);
    assert_eq!(*ev.key(), 0);
    assert_eq!(ev.object_id(), obj.id());
    assert_eq!(ev.service_id(svc1.id().uuid), svc1.id());
    assert_eq!(ev.service_id(svc2.id().uuid), svc2.id());

    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.finished());

    discoverer.stop().await.unwrap();
    discoverer.start_current_only().await.unwrap();
    assert!(!discoverer.finished());

    client.join().await;
    broker.join().await;
}
