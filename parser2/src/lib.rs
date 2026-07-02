#![deny(missing_debug_implementations)]

mod diag;
mod error;
mod fmt;
mod issues;
mod lexer;
mod parser;
mod resolver;
mod span;
#[cfg(test)]
mod test;
mod validate;
mod warning;

pub mod ast;
pub mod visitor;

pub use ast::Schema;
pub use diag::{Diagnostic, DiagnosticKind, DiagnosticRenderer};
pub use error::Error;
pub use fmt::Formatter;
pub use parser::{Parser, ParserEntry, SchemaRef};
pub use resolver::{FilesystemResolver, MemoryResolver, Resolver, SchemaFile};
pub use span::{Span, Spanned};
pub use visitor::Visitor;
pub use warning::Warning;
