use crate::{diag, CommonArgs, CommonReadArgs};
use aldrin_parser::Parser;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct CheckArgs {
    #[structopt(flatten)]
    common_args: CommonArgs,

    #[structopt(flatten)]
    common_read_args: CommonReadArgs,

    /// Path to an Aldrin schema file
    #[structopt(name = "schema")]
    file: PathBuf,
}

pub fn run(args: CheckArgs) -> Result<(), ()> {
    let mut parser = Parser::new();

    for include in args.common_read_args.include {
        parser.add_schema_path(include);
    }

    let parsed = parser.parse(args.file);
    diag::print_diagnostics(&parsed, args.common_args.color).ok();

    if parsed.errors().is_empty() {
        if parsed.warnings().is_empty() {
            println!("No issues found.");
        } else {
            println!("Some warning(s) found.");
        }

        Ok(())
    } else {
        println!("Some error(s) found.");
        Err(())
    }
}
