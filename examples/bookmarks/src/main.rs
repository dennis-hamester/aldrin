mod v1;
mod v2;

use aldrin::core::tokio::TokioTransport;
use aldrin::{Client, Handle};
use anyhow::{anyhow, Context, Result};
use bookmarks_v2::{Bookmarks, BookmarksProxy};
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

    /// Run client and server commands with the old V1 API.
    #[clap(subcommand)]
    V1(v1::Args),

    /// Run client and server commands with the new V2 API.
    #[clap(subcommand)]
    V2(v2::Args),
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
    }

    println!("Shutting down connection to the broker.");

    handle.shutdown();
    join.await
        .with_context(|| anyhow!("failed to shut down client"))?
        .with_context(|| anyhow!("failed to shut down client"))?;

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
        let id = event.service_id(&discoverer, Bookmarks::UUID);
        let bookmarks = BookmarksProxy::new(bus, id).await?;

        let id = bookmarks.id().object_id.uuid;
        let version = bookmarks.version();
        println!("Found list {id} with version {version}.");

        found = true;
    }

    if !found {
        println!("No lists found.");
    }

    Ok(())
}
