use crate::{diag, CommonArgs, CommonReadArgs};
use aldrin_parser::Parser;
use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
pub struct CheckArgs {
    #[clap(flatten)]
    common_args: CommonArgs,

    #[clap(flatten)]
    common_read_args: CommonReadArgs,

    /// Paths to one or more Aldrin schema files
    #[clap(name = "schema", required = true)]
    files: Vec<PathBuf>,
}

pub fn run(args: CheckArgs) -> Result<(), ()> {
    let mut parser = Parser::new();

    for include in args.common_read_args.include {
        parser.add_schema_path(include);
    }

    let mut res = Ok(());
    let mut first = true;
    for file in &args.files {
        if args.files.len() > 1 {
            if first {
                first = false;
            } else {
                println!();
            }
            println!("{}:", file.display());
        }

        let parsed = parser.parse(file);
        diag::print_diagnostics(&parsed, args.common_args.color).ok();

        if parsed.errors().is_empty() {
            if parsed.warnings().is_empty() && parsed.other_warnings().is_empty() {
                println!("No issues found.");
            } else {
                println!("Some warning(s) found.");
            }
        } else {
            println!("Some error(s) found.");
            res = Err(());
        }
    }

    res
}
