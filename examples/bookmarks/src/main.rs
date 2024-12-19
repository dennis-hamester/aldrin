mod old;

use aldrin::core::tokio::TokioTransport;
use aldrin::Client;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpStream;

const BUS_DEFAULT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 24940);

// The generated code can be inspected using rustdoc. Just run
// `cargo doc --document-private-items --open` and look at the `echo` module.
aldrin::generate!(
    "src/bookmarks_old.aldrin",
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
    #[clap(subcommand)]
    Old(old::Args),
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
        bookmarks_old::register_introspection(&handle)?;
        handle.submit_introspection()?;
    }

    match args.cmd {
        Command::Old(args) => old::run(args, &handle).await?,
    }

    println!("Shutting down connection to the broker.");

    handle.shutdown();
    join.await
        .with_context(|| anyhow!("failed to shut down client"))?
        .with_context(|| anyhow!("failed to shut down client"))?;

    Ok(())
}
