use super::Error;
use crate::ast::{Ident, StructField};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};

#[derive(Debug)]
pub struct DuplicateStructFallbackName {
    schema_name: String,
    fallback: Ident,
    first: Span,
    struct_span: Span,
    struct_ident: Option<Ident>,
}

impl DuplicateStructFallbackName {
    pub(crate) fn validate(
        fallback: &Ident,
        fields: &[StructField],
        struct_span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
        for field in fields {
            if fallback.value() == field.name().value() {
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    fallback: fallback.clone(),
                    first: field.name().span(),
                    struct_span,
                    struct_ident: ident.cloned(),
                });

                return;
            }
        }
    }

    pub fn fallback(&self) -> &Ident {
        &self.fallback
    }

    pub fn first(&self) -> Span {
        self.first
    }

    pub fn struct_span(&self) -> Span {
        self.struct_span
    }

    pub fn struct_ident(&self) -> Option<&Ident> {
        self.struct_ident.as_ref()
    }
}

impl Diagnostic for DuplicateStructFallbackName {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = if let Some(ref ident) = self.struct_ident {
            Formatter::new(
                self,
                format!(
                    "fallback field of struct `{}` uses duplicate name `{}`",
                    ident.value(),
                    self.fallback.value()
                ),
            )
        } else {
            Formatter::new(
                self,
                format!(
                    "fallback field of inline struct uses duplicate name `{}`",
                    self.fallback.value()
                ),
            )
        };

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.fallback.span().from,
                self.fallback.span(),
                "fallback field defined here",
            )
            .info_block(schema, self.first.from, self.first, "name first used here");
        }

        fmt.format()
    }
}

impl From<DuplicateStructFallbackName> for Error {
    fn from(e: DuplicateStructFallbackName) -> Self {
        Self::DuplicateStructFallbackName(e)
    }
}
