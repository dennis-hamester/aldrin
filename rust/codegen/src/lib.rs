#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_debug_implementations)]

mod error;
mod schema;

#[cfg(feature = "rust")]
pub mod rust;

use schema::{ModuleName, Schema};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use error::{Error, ErrorKind};

#[derive(Debug)]
pub struct Generator {
    schema: Schema,
    options: Options,
    imported: HashMap<ModuleName, Schema>,
}

impl Generator {
    pub fn from_path<P>(path: P, options: Options) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Generator {
            schema: Schema::parse_file(path)?,
            options,
            imported: HashMap::new(),
        })
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    #[cfg(feature = "rust")]
    pub fn generate_rust(
        &self,
        rust_options: rust::RustOptions,
    ) -> Result<rust::RustOutput, Error> {
        rust::generate(&self.schema, &self.options, rust_options)
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Options {
    pub include_dirs: Vec<PathBuf>,
    pub client: bool,
    pub server: bool,
}

impl Options {
    pub fn new() -> Self {
        Options {
            include_dirs: Vec::new(),
            client: true,
            server: true,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Options::new()
    }
}
