#![deny(missing_debug_implementations)]

mod diag;
mod error;
mod grammar;
mod issues;
mod parser;
mod resolver;
mod schema;
mod span;
#[cfg(test)]
mod test;
mod util;
mod validate;
mod warning;

pub mod ast;

pub use diag::{Diagnostic, DiagnosticKind, Renderer};
pub use error::Error;
pub use parser::Parser;
pub use resolver::{FilesystemResolver, MemoryResolver, Resolver, SchemaFile};
pub use schema::Schema;
pub use span::Span;
pub use warning::Warning;
