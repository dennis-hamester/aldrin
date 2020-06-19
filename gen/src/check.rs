use super::CommonReadArgs;
use aldrin_parser::Parser;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct CheckArgs {
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
    if parsed.errors().is_empty() {
        Ok(())
    } else {
        Err(())
    }
}
