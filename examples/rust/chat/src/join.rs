use super::list::query_rooms;
use super::{chat, JoinArgs};
use aldrin_client::{Client, ObjectUuid};
use aldrin_codec::{JsonSerializer, LengthPrefixed, NoopFilter, TokioCodec};
use anyhow::Result;
use crossterm::{cursor, style, terminal};
use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::stream::StreamExt;
use linefeed::{DefaultTerminal, Interface, ReadResult};
use std::cmp::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::select;

pub(crate) async fn run(args: JoinArgs) -> Result<()> {
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
    let handle = client.handle().clone();
    println!("Connection to broker at {} established.", addr);

    let client_join = {
        tokio::spawn(async move {
            if let Err(e) = client.run().await {
                println!("Error on connection to broker at {}: {}.", addr, e);
            }
        })
    };

    let rooms = query_rooms(&handle).await?;
    let id = match args.room {
        Some(service_uuid) => rooms
            .keys()
            .find(|id| id.object_id.uuid.0 == service_uuid)
            .copied(),

        None => match rooms.len().cmp(&1) {
            Ordering::Equal => rooms.keys().next().copied(),
            Ordering::Greater => {
                println!("The broker hosts multiple chat rooms and none was selected with --room.");
                return Ok(());
            }
            Ordering::Less => {
                println!("No chat rooms are hosted on the broker.");
                return Ok(());
            }
        },
    };
    let id = match id {
        Some(id) => id,
        None => {
            println!("Chat room {} not found on broker.", args.room.unwrap());
            return Ok(());
        }
    };

    let ident_obj = handle.create_object(ObjectUuid::new_v4()).await?;
    let room = chat::ChatProxy::bind(handle.clone(), id).await?;
    let mut room_events = room.events();
    room_events.subscribe_all().await?;

    let cookie = match room
        .join(
            chat::ChatJoinArgs::builder()
                .set_object(ident_obj.id())
                .set_name(args.name.clone())
                .build()?,
        )?
        .await?
    {
        Ok(cookie) => cookie,
        Err(chat::ChatJoinError::InvalidObject) => panic!(),
        Err(chat::ChatJoinError::DuplicateName) => {
            println!("The name {} is already taken.", args.name);
            return Ok(());
        }
    };
    println!("Joined chat room {}.", room.get_name()?.await?);

    let interface = Arc::new(linefeed::Interface::new("chat")?);
    interface.set_prompt(&format!(
        "{}{}{}: ",
        style::SetForegroundColor(style::Color::Magenta),
        args.name,
        style::ResetColor
    ))?;

    {
        let mut writer = interface.lock_writer_erase()?;
        crossterm::queue!(
            writer,
            crossterm::style::Print("\n\n\n"),
            crossterm::cursor::MoveUp(1),
        )?;
    }

    let (sender, mut receiver) = unbounded();
    let linefeed_join = {
        let interface = interface.clone();
        thread::spawn(|| run_linefeed(interface, sender).map_err(|_| ()))
    };

    loop {
        select! {
            msg = receiver.next() => {
                let msg = match msg {
                    Some(msg) => msg,
                    None => break,
                };

                if !msg.is_empty() {
                    room.send(
                        chat::ChatSendArgs::builder()
                            .set_cookie(cookie)
                            .set_message(msg)
                            .build()?,
                    )?
                    .await?
                    .unwrap();
                }

                let mut writer = interface.lock_writer_erase()?;
                crossterm::queue!(
                    writer,
                    cursor::MoveUp(1),
                    terminal::Clear(terminal::ClearType::CurrentLine),
                )?;
            }

            event = room_events.next() => {
                let event = match event {
                    Some(event) => event,
                    None => break,
                };

                match event {
                    chat::ChatEvent::Joined(name) => {
                        let mut writer = interface.lock_writer_erase()?;

                        crossterm::queue!(
                            writer,
                            cursor::MoveUp(1),
                            style::SetForegroundColor(style::Color::Magenta),
                            style::Print(name),
                            style::ResetColor,
                            style::Print(" joined the chat room.\n\n\n"),
                            cursor::MoveUp(1),
                        )?;
                    }

                    chat::ChatEvent::Left(name) => {
                        let mut writer = interface.lock_writer_erase()?;

                        crossterm::queue!(
                            writer,
                            cursor::MoveUp(1),
                            style::SetForegroundColor(style::Color::Magenta),
                            style::Print(name),
                            style::ResetColor,
                            style::Print(" left the chat room.\n\n\n"),
                            cursor::MoveUp(1),
                        )?;
                    }

                    chat::ChatEvent::Message(args) => {
                        let mut writer = interface.lock_writer_erase()?;
                        let sender = args.sender;
                        let message = args.message;

                        crossterm::queue!(
                            writer,
                            cursor::MoveUp(1),
                            style::SetForegroundColor(style::Color::Magenta),
                            style::Print(sender),
                            style::ResetColor,
                            style::Print(format!(": {}\n\n\n", message)),
                            cursor::MoveUp(1),
                        )?;
                    }
                }
            }

            else => break,
        }
    }

    receiver.close();
    interface.cancel_read_line()?;
    linefeed_join.join().ok();
    handle.shutdown();
    client_join.await?;

    Ok(())
}

fn run_linefeed(
    interface: Arc<Interface<DefaultTerminal>>,
    sender: UnboundedSender<String>,
) -> Result<()> {
    loop {
        if sender.is_closed() {
            return Ok(());
        }

        match interface.read_line_step(Some(Duration::from_secs(1)))? {
            Some(ReadResult::Input(msg)) => sender.unbounded_send(msg)?,
            Some(ReadResult::Eof) => return Ok(()),
            Some(ReadResult::Signal(_)) => {}
            None => {}
        }
    }
}
