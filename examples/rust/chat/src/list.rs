// Copyright (c) 2020 Dennis Hamester <dennis.hamester@gmail.com>
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
    let client = Client::connect::<Box<dyn Error>>(t, FIFO_SIZE, FIFO_SIZE).await?;
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
