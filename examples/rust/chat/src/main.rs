mod broker;
mod host;
mod join;
mod list;

aldrin_codegen_macros::generate!("../../schemas/chat.aldrin", warnings_as_errors = true);

use clap::{AppSettings, Clap};
use std::error::Error;
use std::net::SocketAddr;
use uuid::Uuid;

#[derive(Clap)]
struct BrokerArgs {
    /// IP address and port
    #[clap(default_value = "127.0.0.1:5000", name = "endpoint")]
    bind: SocketAddr,
}

#[derive(Clap)]
struct HostArgs {
    /// IP address and port of the broker
    #[clap(short, long, default_value = "127.0.0.1:5000", name = "endpoint")]
    broker: SocketAddr,

    /// Name of the chat room
    name: String,
}

#[derive(Clap)]
struct ListArgs {
    /// IP address and port of the broker
    #[clap(default_value = "127.0.0.1:5000", name = "endpoint")]
    broker: SocketAddr,
}

#[derive(Clap)]
struct JoinArgs {
    /// IP address and port of the broker
    #[clap(short, long, default_value = "127.0.0.1:5000", name = "endpoint")]
    broker: SocketAddr,

    /// UUID of the chat room
    ///
    /// If the UUID is not specified and the broker hosts only a single chat room, then that one
    /// will be used.
    #[clap(short, long)]
    room: Option<Uuid>,

    /// The name under which you will appear
    name: String,
}

#[derive(Clap)]
#[clap(
    global_setting = AppSettings::ColoredHelp,
    global_setting = AppSettings::VersionlessSubcommands,
    global_setting = AppSettings::DisableVersion,
)]
enum Args {
    /// Runs an Aldrin broker on which chat rooms can be hosted
    Broker(BrokerArgs),

    /// Hosts a chat room
    Host(HostArgs),

    /// Lists all chat rooms on a broker
    List(ListArgs),

    /// Joins a chat room
    Join(JoinArgs),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    match Args::parse() {
        Args::Broker(args) => broker::run(args).await?,
        Args::Host(args) => host::run(args).await?,
        Args::List(args) => list::run(args).await?,
        Args::Join(args) => join::run(args).await?,
    };

    Ok(())
}
