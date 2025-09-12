use crate::{diag, CommonGenArgs, CommonReadArgs};
use aldrin_codegen::{Generator, Options, RustOptions};
use aldrin_parser::{FilesystemResolver, Parser};
use anyhow::{anyhow, Context, Result};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(clap::Parser)]
#[clap(arg_required_else_help = true)]
pub(crate) struct RustArgs {
    #[clap(flatten)]
    common_read_args: CommonReadArgs,

    #[clap(flatten)]
    common_gen_args: CommonGenArgs,

    /// Path to a patch to apply to the generated code.
    ///
    /// This argument can be specified multiple times to apply more than one patch.
    #[clap(short, long, number_of_values = 1)]
    patch: Vec<PathBuf>,

    /// Guard introspection code by the specified Cargo feature.
    #[clap(long, value_name = "FEATURE")]
    introspection_if: Option<String>,

    /// Path of the aldrin crate
    #[clap(long = "crate", value_name = "PATH")]
    krate: Option<String>,

    /// Path to an Aldrin schema file.
    schema: PathBuf,
}

pub(crate) fn run(args: RustArgs) -> Result<bool> {
    let output_dir = match args.common_gen_args.output_dir {
        Some(output_dir) => output_dir,

        None => {
            env::current_dir().with_context(|| anyhow!("failed to determine current directory"))?
        }
    };

    let parser = Parser::parse(FilesystemResolver::with_include_paths(
        args.schema,
        args.common_read_args.include,
    ));

    diag::print_diagnostics(&parser);

    if parser.errors().is_empty() {
        if !parser.warnings().is_empty() || !parser.other_warnings().is_empty() {
            println!("Some warning(s) found.");
        }
    } else {
        println!("Some error(s) found.");
        return Ok(false);
    }

    let mut options = Options::new();
    options.client = !args.common_gen_args.no_client;
    options.server = !args.common_gen_args.no_server;
    options.introspection = args.common_gen_args.introspection || args.introspection_if.is_some();

    let mut rust_options = RustOptions::new();
    for patch in &args.patch {
        rust_options.patches.push(patch);
    }
    rust_options.introspection_if = args.introspection_if.as_deref();

    rust_options.krate = args.krate.as_deref();

    let generator = Generator::new(&options, &parser);
    let output = generator.generate_rust(&rust_options)?;

    let module_path = output_dir.join(format!("{}.rs", output.module_name));
    let file = if args.common_gen_args.overwrite {
        File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&module_path)
    } else {
        File::options()
            .create_new(true)
            .write(true)
            .open(&module_path)
    };
    let mut file = file.with_context(|| anyhow!("failed to open `{}`", module_path.display()))?;

    file.write_all(output.module_content.as_bytes())?;
    println!("File `{}` written.", module_path.display());
    Ok(true)
}
