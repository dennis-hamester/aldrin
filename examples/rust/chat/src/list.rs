use super::{chat, ListArgs, FIFO_SIZE};
use aldrin_client::{Client, Handle, ServiceId, SubscribeMode};
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
    let client = Client::connect(t, FIFO_SIZE, FIFO_SIZE).await?;
    println!("Connection to broker at {} established.", addr);

    let mut handle = client.handle().clone();
    let join = {
        tokio::spawn(async move {
            if let Err(e) = client.run::<Box<dyn Error>>().await {
                println!("Error on connection to broker at {}: {}.", addr, e);
            }
        })
    };

    let rooms = query_rooms(&mut handle).await?;
    println!();
    if rooms.is_empty() {
        println!("No chat rooms available.");
    } else {
        println!("Available chat room(s):");
        for (id, name) in rooms {
            println!("{} ({})", name, id.object_id.uuid);
        }
    }

    handle.shutdown().await?;
    join.await?;
    Ok(())
}

pub(crate) async fn query_rooms(
    handle: &mut Handle,
) -> Result<HashMap<ServiceId, String>, Box<dyn Error>> {
    let mut ids = handle.services_created(SubscribeMode::CurrentOnly).await?;
    let mut res = HashMap::new();

    while let Some(id) = ids.next().await {
        if id.uuid != chat::CHAT_UUID {
            continue;
        }

        let mut room = chat::ChatProxy::bind(handle.clone(), id)?;
        let name = room.get_name().await?;
        res.insert(id, name);
    }

    Ok(res)
}
