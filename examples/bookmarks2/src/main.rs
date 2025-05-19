mod v1;
mod v2;
mod v3;

use aldrin::core::tokio::TokioTransport;
use aldrin::core::ObjectUuid;
use aldrin::{Client, Handle};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpStream;

const BUS_DEFAULT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 24940);

// The generated code can be inspected using rustdoc. Just run
// `cargo doc --document-private-items --open` and look at the `bookmarks_v1` and `bookmarks_v2`
// modules.
aldrin::generate!(
    "src/bookmarks_v1.aldrin",
    "src/bookmarks_v2.aldrin",
    "src/bookmarks_v3.aldrin",
    introspection_if = "introspection",
    warnings_as_errors = true,
);

/// Bookmarks example.
#[derive(Parser)]
struct Args {
    /// Address of the broker to connect to.
    #[clap(short, long, default_value_t = BUS_DEFAULT)]
    bus: SocketAddr,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
enum Command {
    /// List all servers and their version.
    List,

    /// Run client and server commands with the V1 API.
    #[clap(subcommand)]
    V1(v1::Args),

    /// Run client and server commands with the V2 API.
    #[clap(subcommand)]
    V2(v2::Args),

    /// Run client and server commands with the V3 API.
    #[clap(subcommand)]
    V3(v3::Args),
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
    /// If this specified, then incompatible calls may be made on the server.
    #[clap(short, long)]
    ignore_version: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("Connecting to broker at {}.", args.bus);

    let stream = TcpStream::connect(&args.bus)
        .await
        .with_context(|| anyhow!("failed to connect to broker at {}", args.bus))?;

    // Setting nodelay on the TCP socket can vastly improve latencies as Aldrin messages are
    // typically small.
    stream.set_nodelay(true)?;

    let transport = TokioTransport::new(stream);

    let client = Client::connect(transport)
        .await
        .with_context(|| anyhow!("failed to connect to broker at {}", args.bus))?;
    let handle = client.handle().clone();
    let join = tokio::spawn(client.run());

    #[cfg(feature = "introspection")]
    {
        bookmarks_v1::register_introspection(&handle)?;
        bookmarks_v2::register_introspection(&handle)?;
        handle.submit_introspection()?;
    }

    match args.cmd {
        Command::List => list(&handle).await?,
        Command::V1(args) => v1::run(args, &handle).await?,
        Command::V2(args) => v2::run(args, &handle).await?,
        Command::V3(args) => v3::run(args, &handle).await?,
    }

    println!("Shutting down connection to the broker.");

    handle.shutdown();
    join.await
        .with_context(|| anyhow!("failed to shut down client"))?
        .with_context(|| anyhow!("failed to shut down client"))?;

    Ok(())
}

async fn list(bus: &Handle) -> Result<()> {
    #[derive(Copy, Clone, PartialEq, Eq, Hash)]
    enum Version {
        V1,
    }

    let discoverer = bus
        .create_discoverer()
        .any_object_with_services(Version::V1, [bookmarks_v1::Bookmarks::UUID])
        .build_current_only_and_wait()
        .await?;

    let mut found = false;

    for bookmarks in &discoverer {
        let version = match bookmarks.key() {
            Version::V1 => {
                let id = bookmarks.service_id(bookmarks_v1::Bookmarks::UUID);
                bookmarks_v1::BookmarksProxy::new(bus, id).await?.version()
            }
        };

        let id = bookmarks.object_id().uuid;
        println!("Found list {id} with version {version}.");

        found = true;
    }

    if !found {
        println!("No lists found.");
    }

    Ok(())
}
