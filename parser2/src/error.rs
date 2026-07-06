mod import_not_found;
mod io_error;
mod parse_error;

use crate::{Diagnostic, DiagnosticKind, DiagnosticRenderer, Parser, SchemaRef};
use derive_more::{Debug, From};

pub(crate) use import_not_found::ImportNotFound;
pub(crate) use io_error::IoError;
pub(crate) use parse_error::{Expected, ParseError};

#[derive(Debug, Clone, From)]
#[from(forward)]
#[debug("{inner:?}")]
pub struct Error {
    inner: Inner,
}

impl Error {
    pub(crate) fn is_fmt_error(&self) -> bool {
        self.inner.is_fmt_error()
    }
}

impl Diagnostic for Error {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema(&self) -> SchemaRef {
        self.inner.schema()
    }

    fn render(&self, renderer: &DiagnosticRenderer, parser: &Parser) -> String {
        self.inner.render(renderer, parser)
    }
}

#[derive(Debug, Clone, From)]
enum Inner {
    #[debug("{_0:?}")]
    ImportNotFound(ImportNotFound),

    #[debug("{_0:?}")]
    Io(IoError),

    #[debug("{_0:?}")]
    Parser(ParseError),
}

impl Inner {
    fn schema(&self) -> SchemaRef {
        match self {
            Self::ImportNotFound(e) => e.schema(),
            Self::Io(e) => e.schema(),
            Self::Parser(e) => e.schema(),
        }
    }

    fn render(&self, renderer: &DiagnosticRenderer, parser: &Parser) -> String {
        match self {
            Self::ImportNotFound(e) => e.render(renderer, parser),
            Self::Io(e) => e.render(renderer, parser),
            Self::Parser(e) => e.render(renderer, parser),
        }
    }

    fn is_fmt_error(&self) -> bool {
        matches!(self, Self::Io(_) | Self::Parser(_))
    }
}
