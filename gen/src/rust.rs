use super::{CommonGenArgs, CommonReadArgs};
use aldrin_codegen::rust::RustOptions;
use aldrin_codegen::{Generator, Options};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct RustArgs {
    #[structopt(flatten)]
    common_read_args: CommonReadArgs,

    #[structopt(flatten)]
    common_gen_args: CommonGenArgs,

    /// Format output with rustfmt
    ///
    /// The formatting style can be customized with --rustfmt-toml.
    #[structopt(long)]
    format: bool,

    /// Path to rustfmt.toml
    ///
    /// If this argument is not specified, standard rustfmt rules apply regarding its configuration.
    #[structopt(long)]
    rustfmt_toml: Option<PathBuf>,

    /// Path to an Aldrin schema file
    #[structopt(name = "schema")]
    file: PathBuf,
}

pub fn run(args: RustArgs) -> Result<(), ()> {
    let mut options = Options::new();
    options.include_dirs = args.common_read_args.include;
    options.client = args.common_gen_args.client;
    options.server = args.common_gen_args.server;

    let gen = match Generator::from_path(args.file, options) {
        Ok(gen) => gen,
        Err(e) => {
            eprintln!("{}", e);
            return Err(());
        }
    };

    let mut rust_options = RustOptions::new();
    rust_options.rustfmt = args.format;
    rust_options.rustfmt_toml = args.rustfmt_toml;

    let output = match gen.generate_rust(rust_options) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("{}", e);
            return Err(());
        }
    };

    let module_path = args
        .common_gen_args
        .output_dir
        .join(format!("{}.rs", output.module_name));
    let file = if args.common_gen_args.overwrite {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(module_path)
    } else {
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(module_path)
    };
    let mut file = match file {
        Ok(file) => file,
        Err(e) => {
            eprintln!("{}", e);
            return Err(());
        }
    };

    match file.write_all(output.module_content.as_bytes()) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            Err(())
        }
    }
}
