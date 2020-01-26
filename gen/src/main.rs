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

use aldrin_codegen::{Generator, Options};
use std::path::PathBuf;
use std::process;
use structopt::clap::ArgGroup;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
enum Args {
    /// Checks an Aldrin schema for errors
    Check(check::CheckArgs),
}

#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("client_or_server"))]
struct CommonGenArgs {
    /// Additional include directories
    #[structopt(short = "I", long, name = "include_dir")]
    include: Vec<PathBuf>,

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

#[derive(StructOpt, Debug)]
struct CheckArgs {
    #[structopt(flatten)]
    common_gen_args: CommonGenArgs,

    /// Path to a Aldrin schema file
    #[structopt(name = "schema")]
    file: PathBuf,
}

fn check(args: CheckArgs) -> Result<(), ()> {
    let mut options = Options::new();
    options.set_include_dirs(args.common_gen_args.include);
    let gen = Generator::from_path(args.file, options);
    println!("{:#?}", gen);
    Ok(())
}

fn main() {
    let res = match Args::from_args() {
        Args::Check(args) => check(args),
    };

    let exit_code = match res {
        Ok(()) => 0,
        Err(()) => 1,
    };

    process::exit(exit_code);
}
