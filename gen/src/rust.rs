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

use super::{CommonGenArgs, CommonReadArgs};
use aldrin_codegen::rust::RustOptions;
use aldrin_codegen::{Generator, Options};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct RustArgs {
    #[structopt(flatten)]
    common_read_args: CommonReadArgs,

    #[structopt(flatten)]
    common_gen_args: CommonGenArgs,

    /// Path to an Aldrin schema file
    #[structopt(name = "schema")]
    file: PathBuf,
}

pub fn run(args: RustArgs) -> Result<(), ()> {
    let mut options = Options::new();
    options.include_dirs = args.common_read_args.include;

    let gen = match Generator::from_path(args.file, options) {
        Ok(gen) => gen,
        Err(e) => {
            eprintln!("{}", e);
            return Err(());
        }
    };

    let rust_options = RustOptions::new();
    let output = match gen.generate_rust(rust_options) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("{}", e);
            return Err(());
        }
    };

    println!("{}", output.module_content);
    Ok(())
}
