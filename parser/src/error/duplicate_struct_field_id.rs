use super::{Error, ErrorKind};
use crate::ast::{Ident, LitPosInt, StructField};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{util, Parser, Span};

#[derive(Debug)]
pub(crate) struct DuplicateStructFieldId {
    schema_name: String,
    duplicate: LitPosInt,
    first: Span,
    struct_ident: Option<Ident>,
    free_id: u32,
}

impl DuplicateStructFieldId {
    pub(crate) fn validate(fields: &[StructField], ident: Option<&Ident>, validate: &mut Validate) {
        let mut max_id = fields
            .iter()
            .fold(0, |cur, field| match field.id().value().parse() {
                Ok(id) if id > cur => id,
                _ => cur,
            });

        util::find_duplicates(
            fields,
            |field| field.id().value(),
            |duplicate, first| {
                max_id += 1;
                let free_id = max_id;
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.id().clone(),
                    first: first.id().span(),
                    struct_ident: ident.cloned(),
                    free_id,
                })
            },
        );
    }
}

impl Diagnostic for DuplicateStructFieldId {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let title = match self.struct_ident {
            Some(ref ident) => format!(
                "duplicate id `{}` in struct `{}`",
                self.duplicate.value(),
                ident.value()
            ),

            None => format!("duplicate id `{}` in inline struct", self.duplicate.value()),
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

impl From<DuplicateStructFieldId> for Error {
    fn from(e: DuplicateStructFieldId) -> Self {
        Self {
            kind: ErrorKind::DuplicateStructFieldId(e),
        }
    }
}
