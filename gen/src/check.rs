use super::CommonReadArgs;
use aldrin_codegen::{Generator, Options};
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
    let mut options = Options::new();
    options.include_dirs = args.common_read_args.include;

    match Generator::from_path(args.file, options) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            Err(())
        }
    }
}
