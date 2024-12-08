use super::Error;
use crate::ast::{EnumVariant, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};

#[derive(Debug)]
pub struct DuplicateEnumFallbackName {
    schema_name: String,
    fallback: Ident,
    first: Span,
    enum_span: Span,
    enum_ident: Option<Ident>,
}

impl DuplicateEnumFallbackName {
    pub(crate) fn validate(
        fallback: &Ident,
        vars: &[EnumVariant],
        enum_span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
        for var in vars {
            if fallback.value() == var.name().value() {
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    fallback: fallback.clone(),
                    first: var.name().span(),
                    enum_span,
                    enum_ident: ident.cloned(),
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

    pub fn enum_span(&self) -> Span {
        self.enum_span
    }

    pub fn enum_ident(&self) -> Option<&Ident> {
        self.enum_ident.as_ref()
    }
}

impl Diagnostic for DuplicateEnumFallbackName {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = if let Some(ref ident) = self.enum_ident {
            Formatter::new(
                self,
                format!(
                    "fallback variant of enum `{}` uses duplicate name `{}`",
                    ident.value(),
                    self.fallback.value()
                ),
            )
        } else {
            Formatter::new(
                self,
                format!(
                    "fallback variant of inline enum uses duplicate name `{}`",
                    self.fallback.value()
                ),
            )
        };

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.fallback.span().from,
                self.fallback.span(),
                "fallback variant defined here",
            )
            .info_block(schema, self.first.from, self.first, "name first used here");
        }

        fmt.format()
    }
}

impl From<DuplicateEnumFallbackName> for Error {
    fn from(e: DuplicateEnumFallbackName) -> Self {
        Self::DuplicateEnumFallbackName(e)
    }
}
