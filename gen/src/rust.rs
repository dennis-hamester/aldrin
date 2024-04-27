use crate::{diag, CommonGenArgs, CommonReadArgs};
use aldrin_codegen::{Generator, Options, RustOptions};
use aldrin_parser::Parser;
use anyhow::{anyhow, Context, Result};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(clap::Parser)]
#[clap(arg_required_else_help = true)]
pub struct RustArgs {
    #[clap(flatten)]
    common_read_args: CommonReadArgs,

    #[clap(flatten)]
    common_gen_args: CommonGenArgs,

    /// Don't generate builders for structs.
    #[clap(long)]
    no_struct_builders: bool,

    /// Don't annotate structs with non_exhaustive attribute.
    #[clap(long)]
    no_struct_non_exhaustive: bool,

    /// Don't annotate enums with non_exhaustive attribute.
    #[clap(long)]
    no_enum_non_exhaustive: bool,

    /// Don't annotate service event enums with non_exhaustive attribute.
    #[clap(long)]
    no_event_non_exhaustive: bool,

    /// Don't annotate service function enums with non_exhaustive attribute.
    #[clap(long)]
    no_function_non_exhaustive: bool,

    /// Path to a patch to apply to the generated code.
    ///
    /// This argument can be specified multiple times to apply more than one patch.
    #[clap(short, long, number_of_values = 1)]
    patch: Vec<PathBuf>,

    /// Path to an Aldrin schema file.
    schema: PathBuf,
}

pub fn run(args: RustArgs) -> Result<bool> {
    let output_dir = match args.common_gen_args.output_dir {
        Some(output_dir) => output_dir,
        None => {
            env::current_dir().with_context(|| anyhow!("failed to determine current directory"))?
        }
    };

    let mut parser = Parser::new();

    for include in args.common_read_args.include {
        parser.add_schema_path(include);
    }

    let parsed = parser.parse(args.schema);
    diag::print_diagnostics(&parsed);

    if parsed.errors().is_empty() {
        if !parsed.warnings().is_empty() || !parsed.other_warnings().is_empty() {
            println!("Some warning(s) found.");
        }
    } else {
        println!("Some error(s) found.");
        return Ok(false);
    }

    let mut options = Options::new();
    options.client = !args.common_gen_args.no_client;
    options.server = !args.common_gen_args.no_server;
    options.introspection = args.common_gen_args.introspection;

    let mut rust_options = RustOptions::new();
    for patch in &args.patch {
        rust_options.patches.push(patch);
    }
    rust_options.struct_builders = !args.no_struct_builders;
    rust_options.struct_non_exhaustive = !args.no_struct_non_exhaustive;
    rust_options.enum_non_exhaustive = !args.no_enum_non_exhaustive;
    rust_options.event_non_exhaustive = !args.no_event_non_exhaustive;
    rust_options.function_non_exhaustive = !args.no_function_non_exhaustive;

    let gen = Generator::new(&options, &parsed);
    let output = gen.generate_rust(&rust_options)?;

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
