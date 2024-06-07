use crate::core::{ObjectUuid, ServiceUuid};
use aldrin_test::aldrin::low_level::Service;
use aldrin_test::aldrin::{Discoverer, DiscovererEvent, DiscovererEventKind, Object};
use aldrin_test::tokio::TestBroker;
use std::fmt::Debug;
use std::hash::Hash;

fn test_created<Key>(
    discoverer: &Discoverer<Key>,
    event: DiscovererEvent<Key>,
    key: Key,
    object: &Object,
    service1: Option<&Service>,
    service2: Option<&Service>,
) where
    Key: Copy + Eq + Hash + Debug,
{
    assert_eq!(event.kind(), DiscovererEventKind::Created);
    assert_eq!(event.key(), key);
    assert_eq!(event.object_id(), object.id());

    let entry = discoverer.entry(key).unwrap();
    assert_eq!(entry.key(), key);
    assert_eq!(entry.object_id(object.id().uuid), Some(object.id()));

    if let Some(service1) = service1 {
        assert_eq!(
            event.service_id(discoverer, service1.id().uuid),
            service1.id()
        );

        assert_eq!(
            entry.service_id(object.id().uuid, service1.id().uuid),
            Some(service1.id())
        );
    }

    if let Some(service2) = service2 {
        assert_eq!(
            event.service_id(discoverer, service2.id().uuid),
            service2.id()
        );

        assert_eq!(
            entry.service_id(object.id().uuid, service2.id().uuid),
            Some(service2.id())
        );
    }
}

fn test_destroyed<Key>(
    discoverer: &Discoverer<Key>,
    event: DiscovererEvent<Key>,
    key: Key,
    object: &Object,
    service1: Option<&Service>,
    service2: Option<&Service>,
) where
    Key: Copy + Eq + Hash + Debug,
{
    assert_eq!(event.kind(), DiscovererEventKind::Destroyed);
    assert_eq!(event.key(), key);
    assert_eq!(event.object_id(), object.id());

    let entry = discoverer.entry(key).unwrap();
    assert_eq!(entry.key(), key);
    assert_eq!(entry.object_id(object.id().uuid), None);

    if let Some(service1) = service1 {
        assert_eq!(entry.service_id(object.id().uuid, service1.id().uuid), None);
    }

    if let Some(service2) = service2 {
        assert_eq!(entry.service_id(object.id().uuid, service2.id().uuid), None);
    }
}

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

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, None, None);

    assert!(!discoverer.is_finished());
    obj.destroy().await.unwrap();
    let ev = discoverer.next_event().await.unwrap();
    test_destroyed(&discoverer, ev, 0, &obj, None, None);

    assert!(!discoverer.is_finished());
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

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, None, None);

    assert!(!discoverer.is_finished());
    obj.destroy().await.unwrap();
    let ev = discoverer.next_event().await.unwrap();
    test_destroyed(&discoverer, ev, 0, &obj, None, None);

    assert!(!discoverer.is_finished());
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

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, Some(&svc1), Some(&svc2));

    assert!(!discoverer.is_finished());
    obj.destroy().await.unwrap();
    let ev = discoverer.next_event().await.unwrap();
    test_destroyed(&discoverer, ev, 0, &obj, Some(&svc1), Some(&svc2));

    assert!(!discoverer.is_finished());
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

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, Some(&svc1), Some(&svc2));

    assert!(!discoverer.is_finished());
    obj.destroy().await.unwrap();
    let ev = discoverer.next_event().await.unwrap();
    test_destroyed(&discoverer, ev, 0, &obj, Some(&svc1), Some(&svc2));

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc1 = obj.create_service(svc1.id().uuid, 0).await.unwrap();
    let svc2 = obj.create_service(svc2.id().uuid, 0).await.unwrap();

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, Some(&svc1), Some(&svc2));

    assert!(!discoverer.is_finished());
    obj.destroy().await.unwrap();
    let ev = discoverer.next_event().await.unwrap();
    test_destroyed(&discoverer, ev, 0, &obj, Some(&svc1), Some(&svc2));

    assert!(!discoverer.is_finished());
    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn empty() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let mut discoverer = client.create_discoverer::<()>().build().await.unwrap();
    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.is_finished());
    assert!(discoverer.entry(()).is_none());

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

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, Some(&svc1), Some(&svc2));

    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.is_finished());

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn restart_any() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    let mut discoverer = client
        .create_discoverer()
        .any(0, [svc.id().uuid])
        .build()
        .await
        .unwrap();

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, Some(&svc), None);

    assert!(!discoverer.is_finished());
    discoverer.restart().await.unwrap();

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, Some(&svc), None);

    assert!(!discoverer.is_finished());
    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn restart_specific() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    let mut discoverer = client
        .create_discoverer()
        .specific(0, obj.id().uuid, [svc.id().uuid])
        .build()
        .await
        .unwrap();

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, Some(&svc), None);

    assert!(!discoverer.is_finished());
    discoverer.restart().await.unwrap();

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, Some(&svc), None);

    assert!(!discoverer.is_finished());
    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn restart_current_only_any() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let uuid = ServiceUuid::new_v4();

    let mut discoverer = client
        .create_discoverer()
        .any(0, [uuid])
        .build_current_only()
        .await
        .unwrap();

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc = obj.create_service(uuid, 0).await.unwrap();

    assert!(!discoverer.is_finished());
    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.is_finished());

    discoverer.restart_current_only().await.unwrap();

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, Some(&svc), None);

    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.is_finished());

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn restart_current_only_specific() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj_uuid = ObjectUuid::new_v4();
    let svc_uuid = ServiceUuid::new_v4();

    let mut discoverer = client
        .create_discoverer()
        .specific(0, obj_uuid, [svc_uuid])
        .build_current_only()
        .await
        .unwrap();

    let obj = client.create_object(obj_uuid).await.unwrap();
    let svc = obj.create_service(svc_uuid, 0).await.unwrap();

    assert!(!discoverer.is_finished());
    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.is_finished());

    discoverer.restart_current_only().await.unwrap();

    assert!(!discoverer.is_finished());
    let ev = discoverer.next_event().await.unwrap();
    test_created(&discoverer, ev, 0, &obj, Some(&svc), None);

    assert!(discoverer.next_event().await.is_none());
    assert!(discoverer.is_finished());

    client.join().await;
    broker.join().await;
}
