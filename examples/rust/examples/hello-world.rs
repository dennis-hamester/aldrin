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

use aldrin_broker::broker;
use aldrin_examples::Error;
use aldrin_proto::Value;
use aldrin_util::channel::{channel, ClientTransport, ConnectionTransport};
use futures::stream::StreamExt;
use uuid::Uuid;

const FIFO_SIZE: usize = 16;

async fn broker(t: ConnectionTransport) -> Result<(), Error> {
    let broker = broker::Broker::builder().build();
    let mut handle = broker.handle().clone();
    let join_handle = tokio::spawn(broker.run());

    let conn = handle.add_connection(t).establish::<Error>().await?;
    conn.run::<Error>().await?;

    handle.shutdown().await?;
    join_handle.await?;

    Ok(())
}

async fn client(t: ClientTransport) -> Result<(), Error> {
    let client = aldrin_client::Client::builder(t).connect::<Error>().await?;
    let mut handle = client.handle().clone();
    let join_handle = tokio::spawn(client.run::<Error>());

    let oc = handle.objects_created(true).await?;
    tokio::spawn(oc.for_each(|id| {
        async move {
            println!("Object {} created.", id);
        }
    }));

    let od = handle.objects_destroyed().await?;
    tokio::spawn(od.for_each(|id| {
        async move {
            println!("Object {} destroyed.", id);
        }
    }));

    let sc = handle.services_created(true).await?;
    tokio::spawn(sc.for_each(|(obj_id, svc_id)| {
        async move {
            println!("Object {} created service {}.", obj_id, svc_id);
        }
    }));

    let sd = handle.services_destroyed().await?;
    tokio::spawn(sd.for_each(|(obj_id, svc_id)| {
        async move {
            println!("Object {} destroyed service {}.", obj_id, svc_id);
        }
    }));

    let mut obj = handle.create_object(Uuid::new_v4()).await?;
    let mut svc = obj.create_service(Uuid::new_v4()).await?;

    let mut svc_proxy = handle.bind_service_proxy(svc.object_id(), svc.id());
    println!("{:#?}", svc_proxy.call(0, Value::None).await);

    svc.destroy().await?;
    obj.destroy().await?;

    handle.shutdown().await?;
    join_handle.await??;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (conn_transport, client_transport) = channel(FIFO_SIZE);

    let broker = tokio::spawn(broker(conn_transport));
    let client = tokio::spawn(client(client_transport));

    broker.await??;
    client.await??;

    Ok(())
}
