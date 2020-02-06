// Copyright (c) 2019 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use aldrin_broker::Broker;
use aldrin_client::{Client, ObjectUuid, ServiceUuid, SubscribeMode};
use aldrin_util::channel::{channel, ClientTransport, ConnectionTransport};
use futures::stream::StreamExt;
use std::error::Error;
use uuid::Uuid;

const FIFO_SIZE: usize = 16;

async fn broker(t: ConnectionTransport) -> Result<(), Box<dyn Error>> {
    let broker = Broker::new(FIFO_SIZE);
    let mut handle = broker.handle().clone();
    let join_handle = tokio::spawn(broker.run());

    let conn = handle
        .add_connection::<_, Box<dyn Error>>(t, FIFO_SIZE)
        .await?;
    conn.run::<Box<dyn Error>>().await?;

    handle.shutdown().await?;
    join_handle.await?;

    Ok(())
}

async fn client(t: ClientTransport) -> Result<(), Box<dyn Error>> {
    let client = Client::connect::<Box<dyn Error>>(t, FIFO_SIZE, FIFO_SIZE).await?;
    let mut handle = client.handle().clone();
    let join_handle = tokio::spawn(async { client.run::<Box<dyn Error>>().await.unwrap() });

    let oc = handle.objects_created(SubscribeMode::All).await?;
    tokio::spawn(oc.for_each(|id| async move {
        println!("Object {} created.", id.uuid);
    }));

    let od = handle.objects_destroyed().await?;
    tokio::spawn(od.for_each(|id| async move {
        println!("Object {} destroyed.", id.uuid);
    }));

    let sc = handle.services_created(SubscribeMode::All).await?;
    tokio::spawn(sc.for_each(|id| async move {
        println!("Object {} created service {}.", id.object_id.uuid, id.uuid);
    }));

    let sd = handle.services_destroyed().await?;
    tokio::spawn(sd.for_each(|id| async move {
        println!(
            "Object {} destroyed service {}.",
            id.object_id.uuid, id.uuid
        );
    }));

    let mut obj = handle.create_object(ObjectUuid(Uuid::new_v4())).await?;
    let mut svc = obj
        .create_service(ServiceUuid(Uuid::new_v4()), FIFO_SIZE)
        .await?;

    svc.destroy().await?;
    obj.destroy().await?;

    handle.shutdown().await?;
    join_handle.await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (conn_transport, client_transport) = channel(FIFO_SIZE);

    let broker = tokio::spawn(async { broker(conn_transport).await.unwrap() });
    let client = tokio::spawn(async { client(client_transport).await.unwrap() });

    broker.await?;
    client.await?;

    Ok(())
}
