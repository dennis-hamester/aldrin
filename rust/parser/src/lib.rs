#![allow(clippy::large_enum_variant)]
#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_debug_implementations)]

mod grammar;
mod issues;
mod parser;
mod schema;
mod span;
mod util;
mod validate;

pub mod ast;
pub mod diag;
pub mod error;
pub mod warning;

pub use diag::Diagnostic;
pub use error::Error;
pub use parser::{Parsed, Parser};
pub use schema::Schema;
pub use span::{LineCol, Position, Span, SpanLines};
pub use warning::Warning;
