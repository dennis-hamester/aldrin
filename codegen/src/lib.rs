#![deny(missing_debug_implementations)]

mod error;
#[cfg(feature = "rust")]
mod util;

#[cfg(feature = "rust")]
pub mod rust;

use aldrin_parser::Parser;

pub use error::{Error, SubprocessError};
#[cfg(feature = "rust")]
pub use rust::{RustOptions, RustOutput};

#[derive(Debug, Copy, Clone)]
pub struct Generator<'a> {
    options: &'a Options,
    parser: &'a Parser,
}

impl<'a> Generator<'a> {
    pub fn new(options: &'a Options, parser: &'a Parser) -> Self {
        assert!(parser.errors().is_empty());
        Generator { options, parser }
    }

    pub fn options(self) -> &'a Options {
        self.options
    }

    pub fn parser(self) -> &'a Parser {
        self.parser
    }

    #[cfg(feature = "rust")]
    pub fn generate_rust(&self, rust_options: &RustOptions) -> Result<RustOutput, Error> {
        rust::generate(self.parser, self.options, rust_options)
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
        Self {
            client: true,
            server: true,
            introspection: false,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}
