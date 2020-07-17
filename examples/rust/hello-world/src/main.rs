use aldrin_broker::Broker;
use aldrin_client::{Client, ObjectEvent, ObjectUuid, ServiceEvent, ServiceUuid, SubscribeMode};
use aldrin_util::channel::{unbounded, Unbounded};
use futures::stream::StreamExt;
use std::error::Error;
use uuid::Uuid;

async fn broker(t: Unbounded) -> Result<(), Box<dyn Error>> {
    let broker = Broker::new();
    let handle = broker.handle().clone();
    let join_handle = tokio::spawn(broker.run());

    let conn = handle.add_connection(t).await?;
    conn.run().await?;

    handle.shutdown();
    join_handle.await??;

    Ok(())
}

async fn client(t: Unbounded) -> Result<(), Box<dyn Error>> {
    let client = Client::connect(t).await?;
    let handle = client.handle().clone();
    let join_handle = tokio::spawn(async { client.run().await.unwrap() });

    let objs = handle.objects(SubscribeMode::All)?;
    tokio::spawn(objs.for_each(|ev| async move {
        match ev {
            ObjectEvent::Created(id) => println!("Object {} created.", id.uuid),
            ObjectEvent::Destroyed(id) => println!("Object {} destroyed.", id.uuid),
        }
    }));

    let svcs = handle.services(SubscribeMode::All)?;
    tokio::spawn(svcs.for_each(|ev| async move {
        match ev {
            ServiceEvent::Created(id) => {
                println!("Object {} created service {}.", id.object_id.uuid, id.uuid)
            }
            ServiceEvent::Destroyed(id) => println!(
                "Object {} destroyed service {}.",
                id.object_id.uuid, id.uuid
            ),
        }
    }));

    let mut obj = handle.create_object(ObjectUuid::new_v4()).await?;
    let mut svc = obj.create_service(ServiceUuid(Uuid::new_v4())).await?;

    svc.destroy().await?;
    obj.destroy().await?;

    handle.shutdown();
    join_handle.await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (conn_transport, client_transport) = unbounded();

    let broker = tokio::spawn(async { broker(conn_transport).await.unwrap() });
    let client = tokio::spawn(async { client(client_transport).await.unwrap() });

    broker.await?;
    client.await?;

    Ok(())
}
