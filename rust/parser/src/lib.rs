#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_debug_implementations)]

mod grammar;
mod issues;
mod parser;
mod schema;
mod span;

pub mod ast;
pub mod error;
pub mod warning;

pub use error::Error;
pub use parser::{Parsed, Parser};
pub use schema::Schema;
pub use span::{LineCol, Position, Span};
pub use warning::Warning;
