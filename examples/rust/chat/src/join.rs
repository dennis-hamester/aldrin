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

use super::list::query_rooms;
use super::{chat, JoinArgs, FIFO_SIZE};
use aldrin_client::{Client, ObjectUuid};
use aldrin_util::codec::{JsonSerializer, LengthPrefixed, TokioCodec};
use crossterm::{cursor, style, terminal};
use futures::channel::mpsc::{channel, Sender};
use futures::stream::StreamExt;
use linefeed::{DefaultTerminal, Interface, ReadResult};
use std::cmp::Ordering;
use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::select;
use uuid::Uuid;

pub(crate) async fn run(args: JoinArgs) -> Result<(), Box<dyn Error>> {
    let addr = args.broker;
    println!("Connecting to broker at {}.", addr);

    let socket = TcpStream::connect(&addr).await?;
    let t = TokioCodec::new(socket, LengthPrefixed::new(), JsonSerializer::new(true));
    let client = Client::connect::<Box<dyn Error>>(t, FIFO_SIZE, FIFO_SIZE).await?;
    let mut handle = client.handle().clone();
    println!("Connection to broker at {} established.", addr);

    let client_join = {
        tokio::spawn(async move {
            if let Err(e) = client.run::<Box<dyn Error>>().await {
                println!("Error on connection to broker at {}: {}.", addr, e);
            }
        })
    };

    let rooms = query_rooms(&mut handle).await?;
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

    let ident_obj = handle.create_object(ObjectUuid(Uuid::new_v4())).await?;
    let mut room = chat::ChatProxy::bind(handle.clone(), id)?;
    let mut room_events = room.events(FIFO_SIZE);
    room_events.subscribe_joined().await?;
    room_events.subscribe_left().await?;
    room_events.subscribe_message().await?;

    let cookie = match room
        .join(
            chat::ChatJoinArgs::builder()
                .set_object_cookie(ident_obj.id().cookie.0)
                .set_name(args.name.clone())
                .build()?,
        )
        .await?
    {
        Ok(cookie) => cookie,
        Err(chat::ChatJoinError::InvalidObject) => panic!(),
        Err(chat::ChatJoinError::DuplicateName) => {
            println!("The name {} is already taken.", args.name);
            return Ok(());
        }
    };
    println!("Joined chat room {}.", room.get_name().await?);

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

    let (sender, mut receiver) = channel(FIFO_SIZE);
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
                    )
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
    handle.shutdown().await?;
    client_join.await?;

    Ok(())
}

fn run_linefeed(
    interface: Arc<Interface<DefaultTerminal>>,
    mut sender: Sender<String>,
) -> Result<(), Box<dyn Error>> {
    loop {
        if sender.is_closed() {
            return Ok(());
        }

        match interface.read_line_step(Some(Duration::from_secs(1)))? {
            Some(ReadResult::Input(msg)) => sender.try_send(msg)?,
            Some(ReadResult::Eof) => return Ok(()),
            Some(ReadResult::Signal(_)) => {}
            None => {}
        }
    }
}
