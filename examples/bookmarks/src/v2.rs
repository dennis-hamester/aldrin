use crate::bookmarks_v2::{
    Bookmark, Bookmarks, BookmarksEvent, BookmarksFunction, BookmarksGetV2Args, BookmarksProxy,
    BookmarksRemoveV2Args, Error as BookmarkError,
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
pub struct Add {
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
pub struct Get {
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
pub struct GetGroups {
    #[clap(flatten)]
    server: ServerArg,

    #[clap(flatten)]
    ignore: IgnoreVersionArg,
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

pub async fn run(args: Args, bus: &Handle) -> Result<()> {
    match args {
        Args::Add(args) => add(args, bus).await,
        Args::Get(args) => get(args, bus).await,
        Args::GetGroups(args) => get_groups(args, bus).await,
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
            BookmarksFunction::GetV2(call) => self.get_v2(call)?,
            BookmarksFunction::Add(call) => self.add(call)?,
            BookmarksFunction::Remove(call) => self.remove(call)?,
            BookmarksFunction::RemoveV2(call) => self.remove_v2(call)?,
            BookmarksFunction::GetGroups(call) => self.get_groups(call)?,
            BookmarksFunction::UnknownFunction(call) => self.unknown_function(call),
        }

        Ok(())
    }

    fn get(&self, mut call: Call<(), Vec<Bookmark>, Infallible>) -> Result<()> {
        println!("Getting all bookmarks in the unspecified group.");

        let list = self
            .list
            .iter()
            .filter(|b| b.group.is_none())
            .cloned()
            .collect();

        call.ok_ref(&list)?;

        Ok(())
    }

    fn get_v2(
        &self,
        mut call: Call<BookmarksGetV2Args, Vec<Bookmark>, BookmarkError>,
    ) -> Result<()> {
        let args = call.take_args();

        if args.unknown_fields.has_fields_set() {
            println!("Rejecting to get bookmarks due to unknown arguments.");
            call.err(&BookmarkError::UnknownFields)?;
            return Ok(());
        }

        if let Some(ref group) = args.group {
            println!("Getting all bookmarks in the group `{group}`.");
        } else {
            println!("Getting all bookmarks in the unspecified group.");
        }

        let list = self
            .list
            .iter()
            .filter(|b| b.group == args.group)
            .cloned()
            .collect();

        call.ok_ref(&list)?;

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

        if self
            .list
            .iter()
            .any(|b| (b.name == bookmark.name) && (b.group == bookmark.group))
        {
            println!("Rejecting bookmark because the name is used already.");
            call.err(&BookmarkError::DuplicateName)?;
            return Ok(());
        }

        if bookmark.url.is_empty() {
            println!("Rejecting bookmark because the URL is empty.");
            call.err(&BookmarkError::InvalidUrl)?;
            return Ok(());
        }

        if let Some(ref group) = bookmark.group {
            if group.is_empty() {
                println!("Rejecting bookmark because the group is empty.");
                call.err(&BookmarkError::InvalidGroup)?;
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
        call.done()?;

        Ok(())
    }

    fn remove(&mut self, mut call: Call<String, (), BookmarkError>) -> Result<()> {
        let name = call.take_args();

        let Some(idx) = self
            .list
            .iter()
            .position(|b| (b.name == name) && b.group.is_none())
        else {
            println!("Failed to remove unknown bookmark `{name}`");
            call.err(&BookmarkError::InvalidName)?;
            return Ok(());
        };

        let bookmark = self.list.remove(idx);
        self.svc.removed(&bookmark)?;
        call.done()?;

        Ok(())
    }

    fn remove_v2(
        &mut self,
        mut call: Call<BookmarksRemoveV2Args, (), BookmarkError>,
    ) -> Result<()> {
        let args = call.take_args();

        if args.unknown_fields.has_fields_set() {
            println!("Rejecting to remove a bookmark due to unknown arguments.");
            call.err(&BookmarkError::UnknownFields)?;
            return Ok(());
        }

        let Some(idx) = self
            .list
            .iter()
            .position(|b| (b.name == args.name) && (b.group == args.group))
        else {
            println!("Failed to remove unknown bookmark `{}`", args.name);
            call.err(&BookmarkError::InvalidName)?;
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

        call.done()?;
        Ok(())
    }

    fn get_groups(&mut self, mut call: Call<(), Vec<Option<String>>, Infallible>) -> Result<()> {
        let mut groups = self
            .list
            .iter()
            .map(|b| b.group.clone())
            .collect::<Vec<_>>();

        groups.sort_unstable();
        groups.dedup();

        call.ok_ref(&groups)?;
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
            .get_v2(&BookmarksGetV2Args {
                group: Some(group.clone()),
                unknown_fields: UnknownFields::new(),
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
            Ok(BookmarksEvent::Added(ev)) | Ok(BookmarksEvent::AddedV2(ev)) => {
                bookmark_added(ev.into_args())
            }

            Ok(BookmarksEvent::Removed(ev)) | Ok(BookmarksEvent::RemovedV2(ev)) => {
                bookmark_removed(ev.into_args())
            }

            Ok(BookmarksEvent::UnknownEvent(event)) => unknown_event(event),
            Err(e) => invalid_event(e),
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

    if let Some(ref group) = args.group {
        if (bookmarks.version() < 2) && !args.ignore.ignore_version {
            return Err(anyhow!("server doesn't support groups"));
        }

        bookmarks
            .remove_v2(&BookmarksRemoveV2Args {
                name: args.name.clone(),
                group: Some(group.clone()),
                unknown_fields: UnknownFields::new(),
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
            Self::InvalidGroup => write!(f, "invalid group"),
            Self::Unknown(var) => write!(f, "unknown error {}", var.id()),
        }
    }
}

impl StdError for BookmarkError {}
