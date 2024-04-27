#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "rust")]
mod rust;

pub mod error;

use aldrin_parser::Parsed;

pub use error::Error;
#[cfg(feature = "rust")]
pub use rust::{RustOptions, RustOutput};

#[derive(Debug)]
pub struct Generator<'a> {
    options: &'a Options,
    parsed: &'a Parsed,
}

impl<'a> Generator<'a> {
    pub fn new(options: &'a Options, parsed: &'a Parsed) -> Self {
        assert!(parsed.errors().is_empty());
        Generator { options, parsed }
    }

    pub fn options(&self) -> &'a Options {
        self.options
    }

    pub fn parsed(&self) -> &'a Parsed {
        self.parsed
    }

    #[cfg(feature = "rust")]
    pub fn generate_rust(&self, rust_options: &RustOptions) -> Result<RustOutput, Error> {
        rust::generate(self.parsed, self.options, rust_options)
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Options {
    pub client: bool,
    pub server: bool,
    pub introspection: bool,
}

impl Options {
    pub fn new() -> Self {
        Options {
            client: true,
            server: true,
            introspection: false,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Options::new()
    }
}
