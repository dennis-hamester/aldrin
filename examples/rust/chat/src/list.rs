use super::{chat, ListArgs};
use aldrin_client::{Client, Handle, ServiceEvent, ServiceId, SubscribeMode};
use aldrin_util::codec::{JsonSerializer, LengthPrefixed, NoopFilter, TokioCodec};
use anyhow::Result;
use futures::stream::StreamExt;
use std::collections::HashMap;
use tokio::net::TcpStream;

pub(crate) async fn run(args: ListArgs) -> Result<()> {
    let addr = args.broker;
    println!("Connecting to broker at {}.", addr);

    let socket = TcpStream::connect(&addr).await?;
    let t = TokioCodec::new(
        socket,
        LengthPrefixed::default(),
        NoopFilter,
        JsonSerializer::default(),
    );
    let client = Client::connect(t).await?;
    println!("Connection to broker at {} established.", addr);

    let handle = client.handle().clone();
    let join = {
        tokio::spawn(async move {
            if let Err(e) = client.run().await {
                println!("Error on connection to broker at {}: {}.", addr, e);
            }
        })
    };

    let rooms = query_rooms(&handle).await?;
    println!();
    if rooms.is_empty() {
        println!("No chat rooms available.");
    } else {
        println!("Available chat room(s):");
        for (id, name) in rooms {
            println!("{} ({})", name, id.object_id.uuid);
        }
    }

    handle.shutdown();
    join.await?;
    Ok(())
}

pub(crate) async fn query_rooms(handle: &Handle) -> Result<HashMap<ServiceId, String>> {
    let mut svcs = handle.services(SubscribeMode::CurrentOnly)?;
    let mut res = HashMap::new();

    while let Some(ev) = svcs.next().await {
        let id = match ev {
            ServiceEvent::Created(id) => id,
            ServiceEvent::Destroyed(_) => continue,
        };

        if id.uuid != chat::CHAT_UUID {
            continue;
        }

        let room = chat::ChatProxy::bind(handle.clone(), id)?;
        let name = room.get_name()?.await?;
        res.insert(id, name);
    }

    Ok(res)
}
