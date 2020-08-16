use crate::{diag, CommonArgs, CommonGenArgs, CommonReadArgs};
use aldrin_codegen::{Generator, Options, RustOptions};
use aldrin_parser::Parser;
use anyhow::Result;
use clap::Clap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Clap)]
pub struct RustArgs {
    #[clap(flatten)]
    common_args: CommonArgs,

    #[clap(flatten)]
    common_read_args: CommonReadArgs,

    #[clap(flatten)]
    common_gen_args: CommonGenArgs,

    /// Format output with rustfmt
    ///
    /// The formatting style can be customized with --rustfmt-toml.
    #[clap(long)]
    format: bool,

    /// Path to rustfmt.toml
    ///
    /// If this argument is not specified, standard rustfmt rules apply regarding its configuration.
    #[clap(long)]
    rustfmt_toml: Option<PathBuf>,

    /// Path to a patch to apply to the generated code
    ///
    /// If --format is used as well, the patch is applied before formatting the code.
    #[clap(short, long)]
    patch: Option<PathBuf>,

    /// Path to an Aldrin schema file
    #[clap(name = "schema")]
    file: PathBuf,
}

pub fn run(args: RustArgs) -> Result<bool> {
    let mut parser = Parser::new();

    for include in args.common_read_args.include {
        parser.add_schema_path(include);
    }

    let parsed = parser.parse(args.file);
    diag::print_diagnostics(&parsed, args.common_args.color)?;

    if parsed.errors().is_empty() {
        if parsed.warnings().is_empty() && parsed.other_warnings().is_empty() {
            println!("Some warning(s) found.");
        }
    } else {
        println!("Some error(s) found.");
        return Ok(false);
    }

    let mut options = Options::new();
    options.client = !args.common_gen_args.no_client;
    options.server = !args.common_gen_args.no_server;

    let mut rust_options = RustOptions::new();
    rust_options.rustfmt = args.format;
    rust_options.rustfmt_toml = args.rustfmt_toml.as_deref();
    rust_options.patch = args.patch.as_deref();

    let gen = Generator::new(&options, &parsed);
    let output = gen.generate_rust(&rust_options)?;

    let module_path = args
        .common_gen_args
        .output_dir
        .join(format!("{}.rs", output.module_name));
    let mut file = if args.common_gen_args.overwrite {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&module_path)?
    } else {
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&module_path)?
    };

    file.write_all(output.module_content.as_bytes())?;
    println!("File `{}` written.", module_path.display());
    Ok(true)
}
