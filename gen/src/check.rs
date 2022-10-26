use crate::{diag, CommonArgs, CommonReadArgs};
use aldrin_parser::Parser;
use anyhow::Result;
use std::path::PathBuf;

#[derive(clap::Parser)]
#[clap(arg_required_else_help = true)]
pub struct CheckArgs {
    #[clap(flatten)]
    common_args: CommonArgs,

    #[clap(flatten)]
    common_read_args: CommonReadArgs,

    /// Paths to one or more Aldrin schema files.
    #[clap(required = true)]
    schemata: Vec<PathBuf>,
}

pub fn run(args: CheckArgs) -> Result<bool> {
    let mut parser = Parser::new();

    for include in args.common_read_args.include {
        parser.add_schema_path(include);
    }

    let mut res = true;
    let mut first = true;
    for schema in &args.schemata {
        if args.schemata.len() > 1 {
            if first {
                first = false;
            } else {
                println!();
            }
            println!("{}:", schema.display());
        }

        let parsed = parser.parse(schema);
        diag::print_diagnostics(&parsed, args.common_args.color)?;

        if parsed.errors().is_empty() {
            if parsed.warnings().is_empty() && parsed.other_warnings().is_empty() {
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
