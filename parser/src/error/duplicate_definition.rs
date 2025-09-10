use super::Error;
use crate::ast::Ident;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{util, Parsed, Schema, Span};

#[derive(Debug)]
pub struct DuplicateDefinition {
    schema_name: String,
    duplicate: Ident,
    first: Span,
}

impl DuplicateDefinition {
    pub(crate) fn validate(schema: &Schema, validate: &mut Validate) {
        util::find_duplicates(
            schema.definitions(),
            |def| def.name().value(),
            |duplicate, first| {
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.name().clone(),
                    first: first.name().span(),
                });
            },
        );
    }

    pub fn duplicate(&self) -> &Ident {
        &self.duplicate
    }

    pub fn first(&self) -> Span {
        self.first
    }
}

impl Diagnostic for DuplicateDefinition {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report =
            renderer.error(format!("duplicate definition `{}`", self.duplicate.value()));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report
                .snippet(schema, self.duplicate.span(), "duplicate definition")
                .context(schema, self.first, "first defined here");
        }

        report.render()
    }
}

impl From<DuplicateDefinition> for Error {
    fn from(e: DuplicateDefinition) -> Self {
        Self::DuplicateDefinition(e)
    }
}
