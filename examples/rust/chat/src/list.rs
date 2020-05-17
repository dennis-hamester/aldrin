use super::{chat, ListArgs};
use aldrin_client::{Client, Handle, ServiceEvent, ServiceId, SubscribeMode};
use aldrin_util::codec::{JsonSerializer, LengthPrefixed, TokioCodec};
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::error::Error;
use tokio::net::TcpStream;

pub(crate) async fn run(args: ListArgs) -> Result<(), Box<dyn Error>> {
    let addr = args.broker;
    println!("Connecting to broker at {}.", addr);

    let socket = TcpStream::connect(&addr).await?;
    let t = TokioCodec::new(socket, LengthPrefixed::new(), JsonSerializer::new(true));
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

pub(crate) async fn query_rooms(
    handle: &Handle,
) -> Result<HashMap<ServiceId, String>, Box<dyn Error>> {
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
