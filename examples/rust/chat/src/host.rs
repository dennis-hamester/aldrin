use super::{chat, HostArgs};
use aldrin_client::{Client, ObjectEvent, ObjectUuid, SubscribeMode};
use aldrin_util::codec::{JsonSerializer, LengthPrefixed, NoopFilter, TokioCodec};
use anyhow::Result;
use futures::stream::StreamExt;
use std::collections::{HashMap, HashSet};
use tokio::net::TcpStream;
use tokio::select;
use uuid::Uuid;

pub(crate) async fn run(args: HostArgs) -> Result<()> {
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

    let obj = handle.create_object(ObjectUuid::new_v4()).await?;
    let mut room = chat::Chat::create(&obj).await?;
    let emitter = room.event_emitter().unwrap();
    let mut objects = HashSet::new();
    let mut objs = handle.objects(SubscribeMode::All)?;
    let mut members = HashMap::new();

    loop {
        select! {
            Some(ev) = objs.next() => {
                match ev {
                    ObjectEvent::Created(id) => {
                        objects.insert(id.cookie);
                    }

                    ObjectEvent::Destroyed(id) => {
                        objects.remove(&id.cookie);
                        if let Some((&cookie, _)) =
                                members.iter().find(|(_, &(_, cookie))| cookie == id.cookie.0) {
                            let (name, _) = members.remove(&cookie).unwrap();
                            emitter.left(name)?;
                        }
                    }
                }
            }

            Some(call) = room.next() => {
                match call {
                    chat::ChatFunction::GetName(reply) => {
                        reply.ok(args.name.clone())?;
                    }

                    chat::ChatFunction::Join(args, reply) => {
                        if !objects.iter().any(|cookie| cookie.0 == args.object_cookie) {
                            reply.err(chat::ChatJoinError::InvalidObject)?;
                            continue;
                        }

                        if members.iter().any(|(_, &(ref name, _))| name == &args.name) {
                            reply.err(chat::ChatJoinError::DuplicateName)?;
                            continue;
                        }

                        let cookie = Uuid::new_v4();
                        members.insert(cookie, (args.name.clone(), args.object_cookie));
                        reply.ok(cookie)?;
                        emitter.joined(args.name)?;
                    }

                    chat::ChatFunction::Send(args, reply) => {
                        match members.get(&args.cookie) {
                            Some((name, _)) => {
                                reply.ok()?;
                                emitter.message(
                                    chat::ChatMessageEvent::builder()
                                        .set_sender(name.clone())
                                        .set_message(args.message)
                                        .build()?,
                                )?;
                            }

                            None => reply.err(chat::ChatSendError::InvalidCookie)?,
                        }
                    }
                }
            }

            else => break,
        }
    }

    handle.shutdown();
    join.await?;

    Ok(())
}
