use crate::bookmarks_v2::{
    Bookmark, Bookmarks, BookmarksCallHandler, BookmarksEventHandler, BookmarksGetV2Args,
    BookmarksGetV2ArgsRef, BookmarksProxy, BookmarksRemoveV2Args, BookmarksRemoveV2ArgsRef,
    Error as BookmarkError,
};
use aldrin::core::adapters::IterAsVec;
use aldrin::core::{ObjectUuid, UnknownFields};
use aldrin::{Error as AldrinError, Event, Handle, Object, Promise, UnknownCall, UnknownEvent};
use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use clap::Parser;
use std::convert::Infallible;
use std::error::Error as StdError;
use std::fmt;
use tokio::signal;

#[derive(Parser)]
pub(crate) enum Args {
    /// Add a bookmark.
    Add(Add),

    /// Get a list of bookmarks.
    Get(Get),

    /// Get a list of bookmark groups.
    GetGroups(GetGroups),

    /// Listen for events from a server.
    Listen(Listen),

    /// Remove a bookmark.
    Remove(Remove),

    /// Run a bookmarks server.
    Server,
}

#[derive(Parser)]
pub(crate) struct Add {
    /// The name of the bookmark to add.
    name: String,

    /// The URL of the bookmark to add.
    url: String,

    /// Optional group of the bookmark.
    ///
    /// If this is not specified, then the bookmark is added to the unspecified group.
    group: Option<String>,

    #[clap(flatten)]
    server: ServerArg,

    #[clap(flatten)]
    ignore: IgnoreVersionArg,
}

#[derive(Parser)]
pub(crate) struct Get {
    /// Optional group to get bookmarks from.
    ///
    /// If this is not specified, then the unspecified group is queried.
    group: Option<String>,

    #[clap(flatten)]
    server: ServerArg,

    #[clap(flatten)]
    ignore: IgnoreVersionArg,
}

#[derive(Parser)]
pub(crate) struct GetGroups {
    #[clap(flatten)]
    server: ServerArg,

    #[clap(flatten)]
    ignore: IgnoreVersionArg,
}

#[derive(Parser)]
pub(crate) struct Listen {
    /// Also subscribe to unknown events.
    #[clap(short, long)]
    unknown: bool,

    #[clap(flatten)]
    server: ServerArg,
}

#[derive(Parser)]
pub(crate) struct Remove {
    /// The name of the bookmark to remove.
    name: String,

    /// Optional group to remove the bookmark from.
    ///
    /// If this is not specified, then the bookmark is removed from the unspecified group.
    group: Option<String>,

    #[clap(flatten)]
    server: ServerArg,

    #[clap(flatten)]
    ignore: IgnoreVersionArg,
}

#[derive(Parser)]
struct ServerArg {
    /// UUID of the server to use.
    ///
    /// If this is not specified, then the first server is used that is found.
    #[clap(short, long)]
    server: Option<ObjectUuid>,
}

#[derive(Parser)]
struct IgnoreVersionArg {
    /// Ignore the version of the server.
    ///
    /// If this specified, then incompatible call may be made on the server.
    #[clap(short, long)]
    ignore_version: bool,
}

pub(crate) async fn run(args: Args, bus: &Handle) -> Result<()> {
    match args {
        Args::Add(args) => add(args, bus).await,
        Args::Get(args) => get(args, bus).await,
        Args::GetGroups(args) => get_groups(args, bus).await,
        Args::Listen(args) => Listener::new(args, bus).await?.run().await,
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
                call = self.svc.next_call() => {
                    match call {
                        Some(call) => Bookmarks::dispatch_call(call, &mut self).await?,
                        None => break Err(anyhow!("broker shut down")),
                    }
                }

                _ = signal::ctrl_c() => break Ok(()),
            }
        }
    }
}

#[async_trait]
impl BookmarksCallHandler for Server {
    type Error = Error;

    async fn get(&mut self, promise: Promise<Vec<Bookmark>, Infallible>) -> Result<()> {
        println!("Getting all bookmarks in the unspecified group.");

        let list = self.list.iter().filter(|b| b.group.is_none());
        promise.ok(IterAsVec(list))?;

        Ok(())
    }

    async fn get_v2(
        &mut self,
        args: BookmarksGetV2Args,
        promise: Promise<Vec<Bookmark>, BookmarkError>,
    ) -> Result<()> {
        if args.unknown_fields.has_fields_set() {
            println!("Rejecting to get bookmarks due to unknown arguments.");
            promise.err(BookmarkError::UnknownFields)?;
            return Ok(());
        }

        if let Some(ref group) = args.group {
            println!("Getting all bookmarks in the group `{group}`.");
        } else {
            println!("Getting all bookmarks in the unspecified group.");
        }

        let list = self.list.iter().filter(|b| b.group == args.group);
        promise.ok(IterAsVec(list))?;

        Ok(())
    }

    async fn add(&mut self, bookmark: Bookmark, promise: Promise<(), BookmarkError>) -> Result<()> {
        if bookmark.unknown_fields.has_fields_set() {
            println!("Rejecting bookmark because is contains unknown fields.");
            promise.err(BookmarkError::UnknownFields)?;
            return Ok(());
        }

        if bookmark.name.is_empty() {
            println!("Rejecting bookmark because the name is empty.");
            promise.err(BookmarkError::InvalidName)?;
            return Ok(());
        }

        if self
            .list
            .iter()
            .any(|b| (b.name == bookmark.name) && (b.group == bookmark.group))
        {
            println!("Rejecting bookmark because the name is used already.");
            promise.err(BookmarkError::DuplicateName)?;
            return Ok(());
        }

        if bookmark.url.is_empty() {
            println!("Rejecting bookmark because the URL is empty.");
            promise.err(BookmarkError::InvalidUrl)?;
            return Ok(());
        }

        if let Some(ref group) = bookmark.group {
            if group.is_empty() {
                println!("Rejecting bookmark because the group is empty.");
                promise.err(BookmarkError::InvalidGroup)?;
                return Ok(());
            }
        }

        if let Some(ref group) = bookmark.group {
            println!(
                "Bookmark `{}` ({}) added to the group `{group}`.",
                bookmark.name, bookmark.url
            );

            self.svc.added_v2(&bookmark)?;
        } else {
            println!(
                "Bookmark `{}` ({}) added to the unspecified group.",
                bookmark.name, bookmark.url
            );

            self.svc.added(&bookmark)?;
        }

        self.list.push(bookmark);
        promise.done()?;

        Ok(())
    }

    async fn remove(&mut self, name: String, promise: Promise<(), BookmarkError>) -> Result<()> {
        let Some(idx) = self
            .list
            .iter()
            .position(|b| (b.name == *name) && b.group.is_none())
        else {
            println!("Failed to remove unknown bookmark `{name}`");
            promise.err(BookmarkError::InvalidName)?;
            return Ok(());
        };

        let bookmark = self.list.remove(idx);
        self.svc.removed(&bookmark)?;
        promise.done()?;

        Ok(())
    }

    async fn remove_v2(
        &mut self,
        args: BookmarksRemoveV2Args,
        promise: Promise<(), BookmarkError>,
    ) -> Result<()> {
        if args.unknown_fields.has_fields_set() {
            println!("Rejecting to remove a bookmark due to unknown arguments.");
            promise.err(BookmarkError::UnknownFields)?;
            return Ok(());
        }

        let Some(idx) = self
            .list
            .iter()
            .position(|b| (b.name == args.name) && (b.group == args.group))
        else {
            println!("Failed to remove unknown bookmark `{}`", args.name);
            promise.err(BookmarkError::InvalidName)?;
            return Ok(());
        };

        let bookmark = self.list.remove(idx);

        if let Some(ref group) = bookmark.group {
            println!("Bookmark `{}` removed from the group `{group}`.", args.name);
            self.svc.removed_v2(&bookmark)?;
        } else {
            println!(
                "Bookmark `{}` removed from the unspecified group.",
                args.name
            );

            self.svc.removed(&bookmark)?;
        }

        promise.done()?;
        Ok(())
    }

    async fn get_groups(
        &mut self,
        promise: Promise<Vec<Option<String>>, Infallible>,
    ) -> Result<()> {
        let mut groups = self
            .list
            .iter()
            .map(|b| b.group.clone())
            .collect::<Vec<_>>();

        groups.sort_unstable();
        groups.dedup();

        promise.ok(groups)?;
        Ok(())
    }

    async fn unknown_function(&mut self, call: UnknownCall) -> Result<()> {
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

        Ok(())
    }

    async fn invalid_call(&mut self, error: AldrinError) -> Result<()> {
        println!("Received an invalid call: {error}.");
        Ok(())
    }
}

async fn add(args: Add, bus: &Handle) -> Result<()> {
    let bookmarks = get_bookmarks(args.server.server, bus).await?;
    let id = bookmarks.id().object_id.uuid;
    println!("Using server {id}.");

    let bookmark = Bookmark {
        name: args.name,
        url: args.url,
        group: args.group,
        unknown_fields: UnknownFields::new(),
    };

    if let Some(ref group) = bookmark.group {
        if (bookmarks.version() < 2) && !args.ignore.ignore_version {
            return Err(anyhow!("server doesn't support groups"));
        }

        bookmarks.add(&bookmark).await?.into_args()?;
        println!("Bookmark `{}` added to the group `{group}`.", bookmark.name);
    } else {
        bookmarks.add(&bookmark).await?.into_args()?;

        println!(
            "Bookmark `{}` added to the unspecified group.",
            bookmark.name
        );
    }

    Ok(())
}

async fn get(args: Get, bus: &Handle) -> Result<()> {
    let bookmarks = get_bookmarks(args.server.server, bus).await?;
    let id = bookmarks.id().object_id.uuid;
    println!("Using server {id}.");

    let list = if let Some(group) = args.group {
        if (bookmarks.version() < 2) && !args.ignore.ignore_version {
            return Err(anyhow!("server doesn't support groups"));
        }

        let list = bookmarks
            .get_v2(BookmarksGetV2ArgsRef {
                group: Some(&group),
                unknown_fields: (),
            })
            .await?
            .into_args()?;

        if list.is_empty() {
            println!("No bookmarks found in the group `{group}`.");
        } else {
            println!("Bookmarks in the group `{group}`:");
        }

        list
    } else {
        let list = bookmarks.get().await?.into_args()?;

        if list.is_empty() {
            println!("No bookmarks found in the unspecified group.");
        } else {
            println!("Bookmarks in the unspecified group:");
        }

        list
    };

    for bookmark in list {
        println!("  `{}`: {}", bookmark.name, bookmark.url);
    }

    Ok(())
}

async fn get_groups(args: GetGroups, bus: &Handle) -> Result<()> {
    let bookmarks = get_bookmarks(args.server.server, bus).await?;
    let id = bookmarks.id().object_id.uuid;
    println!("Using server {id}.");

    if (bookmarks.version() < 2) && !args.ignore.ignore_version {
        return Err(anyhow!("server doesn't support groups"));
    }

    let groups = bookmarks.get_groups().await?.into_args()?;

    if groups.is_empty() {
        println!("No groups found.");
    } else {
        println!("Groups:");

        for group in groups {
            if let Some(group) = group {
                println!("  - `{group}`");
            } else {
                println!("  - Unspecified");
            }
        }
    }

    Ok(())
}

struct Listener {
    bookmarks: BookmarksProxy,
}

impl Listener {
    async fn new(args: Listen, bus: &Handle) -> Result<Self> {
        let bookmarks = get_bookmarks(args.server.server, bus).await?;

        if args.unknown {
            bookmarks.inner().subscribe_all().await?;
        } else {
            bookmarks.subscribe_all().await?;
        }

        Ok(Self { bookmarks })
    }

    async fn run(mut self) -> Result<()> {
        println!("Using server {}.", self.bookmarks.id().object_id.uuid);

        loop {
            tokio::select! {
                event = self.bookmarks.next_event() => {
                    match event {
                        Some(event) => BookmarksProxy::dispatch_event(event, &mut self).await?,
                        None => break,
                    }
                }

                _ = signal::ctrl_c() => break,
            }
        }

        Ok(())
    }

    fn bookmark_added(bookmark: Bookmark) {
        if let Some(group) = bookmark.group {
            println!(
                "Bookmark `{}` ({}) added to the group `{group}`.",
                bookmark.name, bookmark.url
            );
        } else {
            println!(
                "Bookmark `{}` ({}) added to the unspecified group.",
                bookmark.name, bookmark.url
            );
        }
    }

    fn bookmark_removed(bookmark: Bookmark) {
        if let Some(group) = bookmark.group {
            println!(
                "Bookmark `{}` ({}) removed from the group `{group}`.",
                bookmark.name, bookmark.url
            );
        } else {
            println!(
                "Bookmark `{}` ({}) removed from the unspecified group.",
                bookmark.name, bookmark.url
            );
        }
    }
}

#[async_trait]
impl BookmarksEventHandler for Listener {
    type Error = Error;

    async fn added(&mut self, event: Event<Bookmark>) -> Result<()> {
        Self::bookmark_added(event.into_args());
        Ok(())
    }

    async fn added_v2(&mut self, event: Event<Bookmark>) -> Result<()> {
        Self::bookmark_added(event.into_args());
        Ok(())
    }

    async fn removed(&mut self, event: Event<Bookmark>) -> Result<()> {
        Self::bookmark_removed(event.into_args());
        Ok(())
    }

    async fn removed_v2(&mut self, event: Event<Bookmark>) -> Result<()> {
        Self::bookmark_removed(event.into_args());
        Ok(())
    }

    async fn unknown_event(&mut self, event: UnknownEvent) -> Result<()> {
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

        Ok(())
    }

    async fn invalid_event(&mut self, error: AldrinError) -> Result<()> {
        println!("Received an invalid event: {error}.");
        Ok(())
    }
}

async fn remove(args: Remove, bus: &Handle) -> Result<()> {
    let bookmarks = get_bookmarks(args.server.server, bus).await?;
    let id = bookmarks.id().object_id.uuid;
    println!("Using server {id}.");

    if let Some(ref group) = args.group {
        if (bookmarks.version() < 2) && !args.ignore.ignore_version {
            return Err(anyhow!("server doesn't support groups"));
        }

        bookmarks
            .remove_v2(BookmarksRemoveV2ArgsRef {
                name: &args.name,
                group: Some(&group),
                unknown_fields: (),
            })
            .await?
            .into_args()?;

        println!("Bookmark `{}` removed from the group `{group}`.", args.name);
    } else {
        bookmarks.remove(&args.name).await?.into_args()?;

        println!(
            "Bookmark `{}` removed from the unspecified group.",
            args.name
        );
    }

    Ok(())
}

async fn get_bookmarks(id: Option<ObjectUuid>, bus: &Handle) -> Result<BookmarksProxy> {
    let (_, [id]) = bus
        .find_object_n(id, &[Bookmarks::UUID])
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
            Self::InvalidGroup => write!(f, "invalid group"),
            Self::Unknown(var) => write!(f, "unknown error {}", var.id()),
        }
    }
}

impl StdError for BookmarkError {}
