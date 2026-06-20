use crate::{Diagnostic, DiagnosticKind, DiagnosticRenderer, Parser, SchemaRef};
use derive_more::{Debug, From};

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
enum Inner {}

impl Inner {
    fn schema(&self) -> SchemaRef {
        match *self {}
    }

    fn render(&self, _renderer: &DiagnosticRenderer, _parser: &Parser) -> String {
        match *self {}
    }
}
