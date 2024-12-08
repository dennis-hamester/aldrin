use super::Error;
use crate::ast::{EnumVariant, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};

#[derive(Debug)]
pub struct EmptyEnum {
    schema_name: String,
    span: Span,
    ident: Option<Ident>,
}

impl EmptyEnum {
    pub(crate) fn validate(
        vars: &[EnumVariant],
        fallback: Option<&Ident>,
        span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
        if !vars.is_empty() || fallback.is_some() {
            return;
        }

        validate.add_error(Self {
            schema_name: validate.schema_name().to_owned(),
            span,
            ident: ident.cloned(),
        });
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn ident(&self) -> Option<&Ident> {
        self.ident.as_ref()
    }
}

impl Diagnostic for EmptyEnum {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = if let Some(ref ident) = self.ident {
            Formatter::new(self, format!("empty enum `{}`", ident.value()))
        } else {
            Formatter::new(self, "empty inline enum")
        };

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(schema, self.span.from, self.span, "");
        }

        fmt.note("empty enums are not supported")
            .help("add at least one variant to the enum");
        fmt.format()
    }
}

impl From<EmptyEnum> for Error {
    fn from(e: EmptyEnum) -> Self {
        Self::EmptyEnum(e)
    }
}
