use crate::{diag, CommonReadArgs};
use aldrin_parser::{FilesystemResolver, Parser};
use anyhow::Result;
use std::path::PathBuf;

#[derive(clap::Parser)]
#[clap(arg_required_else_help = true)]
pub(crate) struct CheckArgs {
    #[clap(flatten)]
    common_read_args: CommonReadArgs,

    /// Paths to one or more Aldrin schema files.
    #[clap(required = true)]
    schemas: Vec<PathBuf>,
}

pub(crate) fn run(args: CheckArgs) -> Result<bool> {
    let mut res = true;
    let mut first = true;

    for schema in &args.schemas {
        let parser = Parser::parse(FilesystemResolver::with_include_paths(
            schema,
            &args.common_read_args.include,
        ));

        if args.schemas.len() > 1 {
            if first {
                first = false;
            } else {
                println!();
            }

            println!("{}:", schema.display());
        }

        diag::print_diagnostics(&parser);

        if parser.errors().is_empty() {
            if parser.warnings().is_empty() && parser.other_warnings().is_empty() {
                println!("No issues found.");
            } else {
                println!("Some warning(s) found.");
            }
        } else {
            println!("Some error(s) found.");
            res = false;
        }
    }

    Ok(res)
}
