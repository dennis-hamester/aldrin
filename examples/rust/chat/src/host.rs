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

use super::{chat, HostArgs, FIFO_SIZE};
use aldrin_client::{Client, ObjectUuid, SubscribeMode};
use aldrin_util::codec::{JsonSerializer, LengthPrefixed, TokioCodec};
use futures::stream::StreamExt;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tokio::net::TcpStream;
use tokio::select;
use uuid::Uuid;

pub(crate) async fn run(args: HostArgs) -> Result<(), Box<dyn Error>> {
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

    let mut obj = handle.create_object(ObjectUuid(Uuid::new_v4())).await?;
    let mut room = chat::Chat::create(&mut obj, FIFO_SIZE).await?;
    let mut emitter = room.event_emitter().unwrap();
    let mut objects = HashSet::new();
    let mut oc = handle.objects_created(SubscribeMode::All).await?;
    let mut od = handle.objects_destroyed().await?;
    let mut members = HashMap::new();

    loop {
        select! {
            Some(object_id) = od.next() => {
                objects.remove(&object_id.cookie);
                if let Some((&cookie, _)) = members.iter().find(|(_, &(_, cookie))| cookie == object_id.cookie.0) {
                    let (name, _) = members.remove(&cookie).unwrap();
                    emitter.left(name).await?;
                }
            }

            Some(object_id) = oc.next() => {
                objects.insert(object_id.cookie);
            }

            Some(call) = room.next() => {
                match call {
                    chat::ChatFunctions::GetName(reply) => {
                        reply.ok(args.name.clone()).await?;
                    }

                    chat::ChatFunctions::Join(args, reply) => {
                        if !objects.iter().any(|cookie| cookie.0 == args.object_cookie) {
                            reply.err(chat::ChatJoinError::InvalidObject).await?;
                            continue;
                        }

                        if members.iter().any(|(_, &(ref name, _))| name == &args.name) {
                            reply.err(chat::ChatJoinError::DuplicateName).await?;
                            continue;
                        }

                        let cookie = Uuid::new_v4();
                        members.insert(cookie, (args.name.clone(), args.object_cookie));
                        reply.ok(cookie).await?;
                        emitter.joined(args.name).await?;
                    }

                    chat::ChatFunctions::Send(args, reply) => {
                        match members.get(&args.cookie) {
                            Some((name, _)) => {
                                reply.ok().await?;
                                emitter.message(
                                    chat::ChatMessageEvent::builder()
                                        .set_sender(name.clone())
                                        .set_message(args.message)
                                        .build()?,
                                )
                                .await?;
                            }

                            None => reply.err(chat::ChatSendError::InvalidCookie).await?,
                        }
                    }
                }
            }

            else => break,
        }
    }

    handle.shutdown().await?;
    join.await?;

    Ok(())
}
