use aldrin::core::tokio::TokioTransport;
use aldrin::core::{ObjectUuid, ServiceUuid};
use aldrin::{Client, Handle};
use aldrin_broker::{Broker, BrokerHandle};
use anyhow::Result;
use futures_util::future;
use std::time::Instant;
use tokio::io;
use tokio::task::JoinHandle;
use uuid::{uuid, Uuid};

const MAX_BUF_SIZE: usize = 10240;
const NUM: u32 = 1000;
const DEP: u32 = 100;
const BASE_UUID: Uuid = uuid!("00000000-eefb-489f-9a69-3cbbcc2d05a5");

aldrin::service! {
    service Foo {
        uuid = ServiceUuid(uuid!("861eb1f5-457d-42d2-9fd8-4a865a8d9062"));
        version = 1;

        fn foo @ 1;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let (mut broker, join) = broker();
    let client = connect(&mut broker).await?;

    let start = Instant::now();
    for id in 0..NUM {
        tokio::spawn(server(id, broker.clone()));
    }

    query(&client, NUM - 1).await?;
    let diff = start.elapsed();

    println!("query({NUM}): {}ms", diff.as_millis());

    let stats = broker.take_statistics().await?;
    println!("\n{stats:#?}");

    broker.shutdown().await;
    join.await?;
    Ok(())
}

fn broker() -> (BrokerHandle, JoinHandle<()>) {
    let broker = Broker::new();
    let handle = broker.handle().clone();
    let join = tokio::spawn(broker.run());
    (handle, join)
}

async fn connect(broker: &mut BrokerHandle) -> Result<Handle> {
    let (t1, t2) = io::duplex(MAX_BUF_SIZE);
    let t1 = TokioTransport::new(t1);
    let t2 = TokioTransport::new(t2);

    let (client, conn) = tokio::join!(Client::connect(t1), broker.connect(t2));
    let client = client?;
    let conn = conn?;

    let handle = client.handle().clone();

    tokio::spawn(client.run());
    tokio::spawn(conn.run());

    Ok(handle)
}

async fn server(id: u32, mut broker: BrokerHandle) -> Result<()> {
    let bus = connect(&mut broker).await?;

    let obj = bus.create_object(object_uuid(id)).await?;
    let mut svc = Foo::new(&obj).await?;

    if id >= DEP {
        future::try_join_all((0..DEP).map(|dep| Box::pin(query(&bus, id - dep - 1)))).await?;
    }

    while let Some(Ok(FooCall::Foo(call))) = svc.next_call().await {
        call.done()?;
    }

    Ok(())
}

async fn query(bus: &Handle, id: u32) -> Result<()> {
    let uuid = object_uuid(id);

    let (_, [svc]) = bus
        .wait_for_object_with_services_n(uuid, &[FooProxy::UUID])
        .await?;

    let proxy = FooProxy::new(bus, svc).await?;
    proxy.foo().await?;

    Ok(())
}

fn object_uuid(id: u32) -> ObjectUuid {
    let (_, d2, d3, d4) = BASE_UUID.as_fields();
    ObjectUuid(Uuid::from_fields(id, d2, d3, d4))
}
