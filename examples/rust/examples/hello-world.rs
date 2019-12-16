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

use aldrin::{broker, client};
use aldrin_examples::Error;
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
    let client = client::Client::builder(t).connect::<Error>().await?;
    let mut handle = client.handle().clone();
    let join_handle = tokio::spawn(client.run::<Error>());

    let mut oc = handle.objects_created(true).await?;
    let oc_join_handle = tokio::spawn(async move {
        while let Some(id) = oc.next().await {
            println!("Object created: {}", id);
        }
    });

    let mut od = handle.objects_destroyed().await?;
    let od_join_handle = tokio::spawn(async move {
        while let Some(id) = od.next().await {
            println!("Object destroyed: {}", id);
        }
    });

    let mut obj = handle.create_object(Uuid::new_v4()).await?;
    let mut svc = obj.create_service(Uuid::new_v4()).await?;

    svc.destroy().await?;
    obj.destroy().await?;

    handle.shutdown().await?;
    oc_join_handle.await?;
    od_join_handle.await?;
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
