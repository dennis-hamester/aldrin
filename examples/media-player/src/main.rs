mod client;
mod server;

use aldrin::Client;
use aldrin::core::ObjectUuid;
use aldrin::core::tokio::TokioTransport;
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use media_player::Error;
use std::error::Error as StdError;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpStream;

const BUS_DEFAULT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 24940);

// The generated code can be inspected using rustdoc. Just run
// `cargo doc --document-private-items --open` and look at the `media_player` module.
aldrin::generate!(
    "src/media_player.aldrin",
    introspection_if = "introspection",
    warnings_as_errors = true,
);

/// Media player example.
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
    /// Start a client that listens for state changes.
    Listen(ServerArg),

    /// Pause playback.
    Pause(ServerArg),

    /// Start playback.
    Play(Play),

    /// Resume playback.
    Resume(ServerArg),

    /// Start a media player server.
    Server,

    /// Stop playback.
    Stop(ServerArg),
}

#[derive(Parser)]
pub struct Play {
    /// The title of the track to play.
    title: String,

    /// The duration of the track in seconds.
    ///
    /// If none is specified, then the server will choose a value.
    duration: Option<u32>,

    /// Start the track in paused state.
    #[clap(short, long)]
    paused: bool,

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
        media_player::register_introspection_media_player(&handle)?;
        handle.submit_introspection()?;
    }

    match args.cmd {
        Command::Listen(args) => client::listen(args, &handle).await?,
        Command::Pause(args) => client::pause(args, &handle).await?,
        Command::Play(args) => client::play(args, &handle).await?,
        Command::Resume(args) => client::resume(args, &handle).await?,
        Command::Server => server::run(&handle).await?,
        Command::Stop(args) => client::stop(args, &handle).await?,
    }

    println!("Shutting down connection to the broker.");

    handle.shutdown();
    join.await
        .with_context(|| anyhow!("failed to shut down client"))?
        .with_context(|| anyhow!("failed to shut down client"))?;

    Ok(())
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidTitle => write!(f, "invalid title"),
            Self::NotPlaying => write!(f, "not playing"),
            Self::NotPaused => write!(f, "not paused"),
        }
    }
}

impl StdError for Error {}
