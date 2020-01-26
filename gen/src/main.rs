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

mod check;

use std::path::PathBuf;
use std::process;
use structopt::clap::{AppSettings, ArgGroup};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    author,
    about,
    global_settings = &[AppSettings::VersionlessSubcommands, AppSettings::ColoredHelp]
)]
enum Args {
    /// Checks an Aldrin schema for errors
    Check(check::CheckArgs),
}

#[derive(StructOpt, Debug)]
pub struct CommonReadArgs {
    /// Additional include directories
    #[structopt(short = "I", long, name = "include_dir")]
    include: Vec<PathBuf>,
}

#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("client_or_server"))]
pub struct CommonGenArgs {
    /// Generates only client-side code
    ///
    /// The default is to generate code for both the client and server. This flag and --server-only
    /// are mutually exclusive.
    #[structopt(long, group = "client_or_server")]
    client_only: bool,

    /// Generates only server-side code
    ///
    /// The default is to generate code for both the client and server. This flag and --client-only
    /// are mutually exclusive.
    #[structopt(long, group = "client_or_server")]
    server_only: bool,
}

fn main() {
    let res = match Args::from_args() {
        Args::Check(args) => check::run(args),
    };

    let exit_code = match res {
        Ok(()) => 0,
        Err(()) => 1,
    };

    process::exit(exit_code);
}
