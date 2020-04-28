use super::{chat, HostArgs};
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
    let client = Client::connect(t).await?;
    println!("Connection to broker at {} established.", addr);

    let mut handle = client.handle().clone();
    let join = {
        tokio::spawn(async move {
            if let Err(e) = client.run().await {
                println!("Error on connection to broker at {}: {}.", addr, e);
            }
        })
    };

    let mut obj = handle.create_object(ObjectUuid(Uuid::new_v4())).await?;
    let mut room = chat::Chat::create(&mut obj).await?;
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
                    chat::ChatFunction::GetName(reply) => {
                        reply.ok(args.name.clone()).await?;
                    }

                    chat::ChatFunction::Join(args, reply) => {
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

                    chat::ChatFunction::Send(args, reply) => {
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

    handle.shutdown().await;
    join.await?;

    Ok(())
}
