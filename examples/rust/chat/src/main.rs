// Copyright (c) 2020 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

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

const FIFO_SIZE: usize = 16;

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
