use super::{Error, ErrorKind};
use crate::ast::{EnumVariant, Ident, LitInt};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{Parser, Span, util};

#[derive(Debug)]
pub(crate) struct DuplicateEnumVariantId {
    schema_name: String,
    duplicate: LitInt,
    first: Span,
    enum_ident: Option<Ident>,
    free_id: u32,
}

impl DuplicateEnumVariantId {
    pub(crate) fn validate(vars: &[EnumVariant], ident: Option<&Ident>, validate: &mut Validate) {
        let mut max_id = vars
            .iter()
            .filter_map(|var| var.id().value().parse().ok())
            .max()
            .unwrap_or(0);

        util::find_duplicates(
            vars.iter()
                .filter(|var| var.id().value().parse::<u32>().is_ok()),
            |var| var.id().value(),
            |duplicate, first| {
                max_id += 1;
                let free_id = max_id;
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.id().clone(),
                    first: first.id().span(),
                    enum_ident: ident.cloned(),
                    free_id,
                });
            },
        );
    }
}

impl Diagnostic for DuplicateEnumVariantId {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let title = if let Some(ref ident) = self.enum_ident {
            format!(
                "duplicate id `{}` in enum `{}`",
                self.duplicate.value(),
                ident.value()
            )
        } else {
            format!("duplicate id `{}` in inline enum", self.duplicate.value())
        };

        let mut report = renderer.error(title);

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report
                .snippet(schema, self.duplicate.span(), "duplicate defined here")
                .context(schema, self.first, "first defined here");
        }

        report = report.help(format!("use a free id, e.g. {}", self.free_id));
        report.render()
    }
}

impl From<DuplicateEnumVariantId> for Error {
    fn from(e: DuplicateEnumVariantId) -> Self {
        Self {
            kind: ErrorKind::DuplicateEnumVariantId(e),
        }
    }
}
