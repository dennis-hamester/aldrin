use crate::bookmarks_old::{
    Bookmark, Bookmarks, BookmarksFunction, BookmarksProxy, Error as BookmarkError,
};
use aldrin::core::ObjectUuid;
use aldrin::{Error, Handle, Object, Promise, UnknownCall};
use anyhow::{anyhow, Result};
use clap::Parser;
use std::convert::Infallible;
use tokio::signal;
use uuid::Uuid;

#[derive(Parser)]
pub enum Args {
    Add(Add),
    Get(Get),
    List,
    Server,
}

#[derive(Parser)]
pub struct Add {
    #[clap(long)]
    server: Option<Uuid>,

    name: String,
    url: String,
}

#[derive(Parser)]
pub struct Get {
    #[clap(long)]
    server: Option<Uuid>,
}

pub async fn run(args: Args, bus: &Handle) -> Result<()> {
    match args {
        Args::Add(args) => add(args, bus).await,
        Args::Get(args) => get(args, bus).await,
        Args::List => list(bus).await,
        Args::Server => server(bus).await,
    }
}

async fn add(args: Add, bus: &Handle) -> Result<()> {
    let bookmarks = get_bookmarks(args.server, bus).await?;
    let id = bookmarks.id().object_id.uuid;

    let bookmark = Bookmark {
        name: args.name,
        url: args.url,
    };

    match bookmarks.add(&bookmark).await? {
        Ok(()) => println!("Bookmark `{}` added to {}.", bookmark.name, id),
        Err(BookmarkError::InvalidName) => println!("The name `{}` is invalid.", bookmark.name),
        Err(BookmarkError::InvalidUrl) => println!("The URL `{}` is invalid.", bookmark.url),
        Err(BookmarkError::Unknown(_)) => println!("An unknown error occurred"),
    }

    Ok(())
}

async fn get(args: Get, bus: &Handle) -> Result<()> {
    let bookmarks = get_bookmarks(args.server, bus).await?;
    let list = bookmarks.get().await??;

    if list.is_empty() {
        println!("No bookmarks found on {}.", bookmarks.id().object_id.uuid);
    } else {
        println!("Bookmarks of {}:", bookmarks.id().object_id.uuid);

        for bookmark in list {
            println!("  {}: {}", bookmark.name, bookmark.url);
        }
    }

    Ok(())
}

async fn list(bus: &Handle) -> Result<()> {
    let mut discoverer = bus
        .create_discoverer()
        .any((), [Bookmarks::UUID])
        .build_current_only()
        .await?;

    let mut found = false;

    while let Some(event) = discoverer.next_event().await {
        println!("Found bookmark server {}.", event.object_id().uuid);
        found = true;
    }

    if !found {
        println!("No bookmark servers found.");
    }

    Ok(())
}

async fn server(bus: &Handle) -> Result<()> {
    Server::new(bus).await?.run().await
}

struct Server {
    _obj: Object,
    svc: Bookmarks,
    list: Vec<Bookmark>,
}

impl Server {
    async fn new(bus: &Handle) -> Result<Self> {
        let obj = bus.create_object(ObjectUuid::new_v4()).await?;
        let svc = Bookmarks::new(&obj).await?;
        Ok(Self {
            _obj: obj,
            svc,
            list: Vec::new(),
        })
    }

    async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                func = self.svc.next_call() => {
                    match func {
                        Some(Ok(func)) => self.handle_call(func)?,
                        Some(Err(e)) => self.invalid_call(e),
                        None => break Err(anyhow!("broker shut down")),
                    }
                }

                _ = signal::ctrl_c() => break Ok(()),
            }
        }
    }

    fn handle_call(&self, func: BookmarksFunction) -> Result<()> {
        match func {
            BookmarksFunction::Get(promise) => self.get(promise)?,
            BookmarksFunction::Add(bookmark, promise) => self.add(bookmark, promise),
            BookmarksFunction::Remove(name, promise) => self.remove(name, promise),
            BookmarksFunction::UnknownFunction(call) => self.unknown_function(call),
        }

        Ok(())
    }

    fn get(&self, promise: Promise<Vec<Bookmark>, Infallible>) -> Result<()> {
        promise.ok(&self.list)?;
        Ok(())
    }

    fn add(&self, _bookmark: Bookmark, _promise: Promise<(), BookmarkError>) {
        todo!()
    }

    fn remove(&self, _name: String, _promise: Promise<(), BookmarkError>) {
        todo!()
    }

    fn unknown_function(&self, call: UnknownCall) {
        match call.deserialize_as_value() {
            Ok(args) => println!(
                "Received unknown call {} with arguments {args:?}.",
                call.id()
            ),

            Err(e) => println!(
                "Received unknown call {} with invalid arguments ({e}).",
                call.id()
            ),
        }
    }

    fn invalid_call(&self, e: Error) {
        println!("Received invalid call: {e}.");
    }
}

async fn get_bookmarks(id: Option<Uuid>, bus: &Handle) -> Result<BookmarksProxy> {
    let (_, [id]) = bus
        .find_object(id.map(ObjectUuid), &[Bookmarks::UUID])
        .await?
        .ok_or_else(|| anyhow!("list not found"))?;

    let bookmarks = BookmarksProxy::new(bus, id).await?;
    Ok(bookmarks)
}
