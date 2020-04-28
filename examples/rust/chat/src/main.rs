mod broker;
mod host;
mod join;
mod list;

aldrin_codegen_macros::generate!("../../schemas/chat.aldrin");

use std::error::Error;
use std::net::SocketAddr;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(StructOpt)]
#[structopt(no_version)]
struct BrokerArgs {
    /// IP address and port
    #[structopt(default_value = "127.0.0.1:5000", name = "endpoint")]
    bind: SocketAddr,
}

#[derive(StructOpt)]
#[structopt(no_version)]
struct HostArgs {
    /// IP address and port of the broker
    #[structopt(short, long, default_value = "127.0.0.1:5000", name = "endpoint")]
    broker: SocketAddr,

    /// Name of the chat room
    name: String,
}

#[derive(StructOpt)]
#[structopt(no_version)]
struct ListArgs {
    /// IP address and port of the broker
    #[structopt(default_value = "127.0.0.1:5000", name = "endpoint")]
    broker: SocketAddr,
}

#[derive(StructOpt)]
#[structopt(no_version)]
struct JoinArgs {
    /// IP address and port of the broker
    #[structopt(short, long, default_value = "127.0.0.1:5000", name = "endpoint")]
    broker: SocketAddr,

    /// UUID of the chat room
    ///
    /// If the UUID is not specified and the broker hosts only a single chat room, then that one
    /// will be used.
    #[structopt(short, long)]
    room: Option<Uuid>,

    /// The name under which you will appear
    name: String,
}

#[derive(StructOpt)]
#[structopt(
    global_settings = &[
        AppSettings::VersionlessSubcommands,
        AppSettings::ColoredHelp,
        AppSettings::DisableVersion,
    ],
    no_version,
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
    match Args::from_args() {
        Args::Broker(args) => broker::run(args).await?,
        Args::Host(args) => host::run(args).await?,
        Args::List(args) => list::run(args).await?,
        Args::Join(args) => join::run(args).await?,
    };

    Ok(())
}
