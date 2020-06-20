use super::Error;
use crate::ast::{Ident, StructField};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateStructField {
    schema_name: String,
    duplicate: Ident,
    original_span: Span,
    struct_span: Span,
    struct_ident: Option<Ident>,
}

impl DuplicateStructField {
    pub(crate) fn validate(
        fields: &[StructField],
        struct_span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
        let mut idents = HashMap::new();

        for field in fields {
            match idents.entry(field.name().value()) {
                Entry::Vacant(e) => {
                    e.insert(field.name());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateStructField {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: field.name().clone(),
                        original_span: e.get().span(),
                        struct_span,
                        struct_ident: ident.cloned(),
                    });
                }
            }
        }
    }

    pub fn duplicate(&self) -> &Ident {
        &self.duplicate
    }

    pub fn original_span(&self) -> Span {
        self.original_span
    }

    pub fn struct_span(&self) -> Span {
        self.struct_span
    }

    pub fn struct_ident(&self) -> Option<&Ident> {
        self.struct_ident.as_ref()
    }
}

impl Diagnostic for DuplicateStructField {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = if let Some(ref ident) = self.struct_ident {
            Formatter::error(format!(
                "duplicate field `{}` in struct `{}`",
                self.duplicate.value(),
                ident.value()
            ))
        } else {
            Formatter::error(format!(
                "duplicate field `{}` in inline struct",
                self.duplicate.value()
            ))
        };

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.duplicate.span().from,
                self.duplicate.span(),
                "duplicate defined here",
            )
            .info_block(
                schema,
                self.original_span.from,
                self.original_span,
                "first defined here",
            );

            if let Some(ref ident) = self.struct_ident {
                fmt.info_block(
                    schema,
                    ident.span().from,
                    ident.span(),
                    format!("struct `{}` defined here", ident.value()),
                );
            } else {
                fmt.info_block(
                    schema,
                    self.struct_span.from,
                    self.struct_span,
                    "inline struct defined here",
                );
            }
        }

        fmt.format()
    }
}

impl From<DuplicateStructField> for Error {
    fn from(e: DuplicateStructField) -> Self {
        Error::DuplicateStructField(e)
    }
}
