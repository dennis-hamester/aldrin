mod duplicate_import;
mod unused_import;

use crate::{Diagnostic, DiagnosticKind, DiagnosticRenderer, Parser, SchemaRef};
use derive_more::{Debug, From};

pub(crate) use duplicate_import::DuplicateImport;
pub(crate) use unused_import::UnusedImport;

#[derive(Debug, Clone, From)]
#[from(forward)]
#[debug("{inner:?}")]
pub struct Warning {
    inner: Inner,
}

impl Diagnostic for Warning {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
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
    DuplicateImport(DuplicateImport),

    #[debug("{_0:?}")]
    UnusedImport(UnusedImport),
}

impl Inner {
    fn schema(&self) -> SchemaRef {
        match self {
            Self::DuplicateImport(w) => w.schema(),
            Self::UnusedImport(w) => w.schema(),
        }
    }

    fn render(&self, renderer: &DiagnosticRenderer, parser: &Parser) -> String {
        match self {
            Self::DuplicateImport(w) => w.render(renderer, parser),
            Self::UnusedImport(w) => w.render(renderer, parser),
        }
    }
}
