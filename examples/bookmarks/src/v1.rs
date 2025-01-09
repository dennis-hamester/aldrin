use crate::bookmarks_v1::{
    Bookmark, Bookmarks, BookmarksEvent, BookmarksFunction, BookmarksProxy, Error as BookmarkError,
};
use aldrin::core::{ObjectUuid, UnknownFields};
use aldrin::{Call, Error, Handle, Object, UnknownCall, UnknownEvent};
use anyhow::{anyhow, Result};
use clap::Parser;
use std::convert::Infallible;
use std::error::Error as StdError;
use std::fmt;
use tokio::signal;

#[derive(Parser)]
pub enum Args {
    /// Add a bookmark.
    Add(Add),

    /// Get a list of bookmarks.
    Get(ServerArg),

    /// Listen for events from a server.
    Listen(Listen),

    /// Remove a bookmark.
    Remove(Remove),

    /// Run a bookmarks server.
    Server,
}

#[derive(Parser)]
pub struct Add {
    /// The name of the bookmark to add.
    name: String,

    /// The URL of the bookmark to add.
    url: String,

    #[clap(flatten)]
    server: ServerArg,
}

#[derive(Parser)]
pub struct Listen {
    /// Also subscribe to unknown events.
    #[clap(short, long)]
    unknown: bool,

    #[clap(flatten)]
    server: ServerArg,
}

#[derive(Parser)]
pub struct Remove {
    /// The name of the bookmark to remove.
    name: String,

    #[clap(flatten)]
    server: ServerArg,
}

#[derive(Parser)]
pub struct ServerArg {
    /// UUID of the server to use.
    ///
    /// If this is not specified, then the first server is used that is found.
    #[clap(short, long)]
    server: Option<ObjectUuid>,
}

pub async fn run(args: Args, bus: &Handle) -> Result<()> {
    match args {
        Args::Add(args) => add(args, bus).await,
        Args::Get(args) => get(args.server, bus).await,
        Args::Listen(args) => listen(args, bus).await,
        Args::Remove(args) => remove(args, bus).await,
        Args::Server => Server::new(bus).await?.run().await,
    }
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

    fn handle_call(&mut self, func: BookmarksFunction) -> Result<()> {
        match func {
            BookmarksFunction::Get(call) => self.get(call)?,
            BookmarksFunction::Add(call) => self.add(call)?,
            BookmarksFunction::Remove(call) => self.remove(call)?,
            BookmarksFunction::UnknownFunction(call) => self.unknown_function(call),
        }

        Ok(())
    }

    fn get(&self, mut call: Call<(), Vec<Bookmark>, Infallible>) -> Result<()> {
        println!("Getting all bookmarks.");
        call.ok(&self.list)?;
        Ok(())
    }

    fn add(&mut self, mut call: Call<Bookmark, (), BookmarkError>) -> Result<()> {
        let bookmark = call.take_args();

        if bookmark.unknown_fields.has_fields_set() {
            println!("Rejecting bookmark because is contains unknown fields.");
            call.err(&BookmarkError::UnknownFields)?;
            return Ok(());
        }

        if bookmark.name.is_empty() {
            println!("Rejecting bookmark because the name is empty.");
            call.err(&BookmarkError::InvalidName)?;
            return Ok(());
        }

        if self.list.iter().any(|b| b.name == bookmark.name) {
            println!("Rejecting bookmark because the name is used already.");
            call.err(&BookmarkError::DuplicateName)?;
            return Ok(());
        }

        if bookmark.url.is_empty() {
            println!("Rejecting bookmark because the URL is empty.");
            call.err(&BookmarkError::InvalidUrl)?;
            return Ok(());
        }

        println!("Bookmark `{}` added.", bookmark.name);
        self.svc.added(&bookmark)?;
        self.list.push(bookmark);
        call.done()?;

        Ok(())
    }

    fn remove(&mut self, mut call: Call<String, (), BookmarkError>) -> Result<()> {
        let name = call.take_args();

        let Some(idx) = self.list.iter().position(|b| b.name == name) else {
            println!("Failed to remove unknown bookmark `{name}`");
            call.err(&BookmarkError::InvalidName)?;
            return Ok(());
        };

        println!("Bookmark `{name}` removed.");
        let bookmark = self.list.remove(idx);
        self.svc.removed(&bookmark)?;
        call.done()?;

        Ok(())
    }

    fn unknown_function(&self, call: UnknownCall) {
        match call.deserialize_as_value() {
            Ok(args) => println!(
                "Received an unknown call {} with arguments {args:?}.",
                call.id()
            ),

            Err(e) => println!(
                "Received an unknown call {} with invalid arguments ({e}).",
                call.id()
            ),
        }
    }

    fn invalid_call(&self, e: Error) {
        println!("Received an invalid call: {e}.");
    }
}

async fn add(args: Add, bus: &Handle) -> Result<()> {
    let bookmarks = get_bookmarks(args.server.server, bus).await?;
    let id = bookmarks.id().object_id.uuid;
    println!("Using server {id}.");

    let bookmark = Bookmark {
        name: args.name,
        url: args.url,
        unknown_fields: UnknownFields::new(),
    };

    bookmarks.add(&bookmark).await?.into_args()?;
    println!("Bookmark `{}` added.", bookmark.name);
    Ok(())
}

async fn get(server: Option<ObjectUuid>, bus: &Handle) -> Result<()> {
    let bookmarks = get_bookmarks(server, bus).await?;
    let id = bookmarks.id().object_id.uuid;
    println!("Using server {id}.");

    let list = bookmarks.get().await?.into_args()?;

    if list.is_empty() {
        println!("No bookmarks found.");
    } else {
        println!("Bookmarks:");

        for bookmark in list {
            println!("  `{}`: {}", bookmark.name, bookmark.url);
        }
    }

    Ok(())
}

async fn listen(args: Listen, bus: &Handle) -> Result<()> {
    let mut bookmarks = get_bookmarks(args.server.server, bus).await?;
    let id = bookmarks.id().object_id.uuid;
    println!("Using server {id}.");

    if args.unknown {
        bookmarks.inner().subscribe_all().await?;
    } else {
        bookmarks.subscribe_all().await?;
    }

    while let Some(event) = bookmarks.next_event().await {
        match event {
            Ok(BookmarksEvent::Added(ev)) => bookmark_added(ev.into_args()),
            Ok(BookmarksEvent::Removed(ev)) => bookmark_removed(ev.into_args()),
            Ok(BookmarksEvent::UnknownEvent(ev)) => unknown_event(ev),
            Err(e) => invalid_event(e),
        }
    }

    Ok(())
}

fn bookmark_added(bookmark: Bookmark) {
    println!("Bookmark `{}` ({}) added.", bookmark.name, bookmark.url);
}

fn bookmark_removed(bookmark: Bookmark) {
    println!("Bookmark `{}` ({}) removed.", bookmark.name, bookmark.url);
}

fn unknown_event(event: UnknownEvent) {
    match event.deserialize_as_value() {
        Ok(args) => println!(
            "Received an unknown event {} with arguments {args:?}.",
            event.id()
        ),

        Err(e) => println!(
            "Received an unknown event {} with invalid arguments ({e}).",
            event.id()
        ),
    }
}

fn invalid_event(e: Error) {
    println!("Received an invalid event: {e}.");
}

async fn remove(args: Remove, bus: &Handle) -> Result<()> {
    let bookmarks = get_bookmarks(args.server.server, bus).await?;
    let id = bookmarks.id().object_id.uuid;
    println!("Using server {id}.");

    bookmarks.remove(&args.name).await?.into_args()?;
    println!("Bookmark `{}` removed.", args.name);
    Ok(())
}

async fn get_bookmarks(id: Option<ObjectUuid>, bus: &Handle) -> Result<BookmarksProxy> {
    let (_, [id]) = bus
        .find_object(id, &[Bookmarks::UUID])
        .await?
        .ok_or_else(|| anyhow!("server not found"))?;

    let bookmarks = BookmarksProxy::new(bus, id).await?;
    Ok(bookmarks)
}

impl fmt::Display for BookmarkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidName => write!(f, "invalid name"),
            Self::DuplicateName => write!(f, "duplicate name"),
            Self::InvalidUrl => write!(f, "invalid url"),
            Self::UnknownFields => write!(f, "unknown fields"),
            Self::Unknown(var) => write!(f, "unknown error {}", var.id()),
        }
    }
}

impl StdError for BookmarkError {}
