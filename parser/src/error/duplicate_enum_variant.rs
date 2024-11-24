use super::Error;
use crate::ast::{EnumVariant, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed, Span};

#[derive(Debug)]
pub struct DuplicateEnumVariant {
    schema_name: String,
    duplicate: Ident,
    first: Span,
    enum_span: Span,
    enum_ident: Option<Ident>,
}

impl DuplicateEnumVariant {
    pub(crate) fn validate(
        vars: &[EnumVariant],
        enum_span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
        util::find_duplicates(
            vars,
            |var| var.name().value(),
            |duplicate, first| {
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.name().clone(),
                    first: first.name().span(),
                    enum_span,
                    enum_ident: ident.cloned(),
                })
            },
        );
    }

    pub fn duplicate(&self) -> &Ident {
        &self.duplicate
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

impl Diagnostic for DuplicateEnumVariant {
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
                    "duplicate variant `{}` in enum `{}`",
                    self.duplicate.value(),
                    ident.value()
                ),
            )
        } else {
            Formatter::new(
                self,
                format!(
                    "duplicate variant `{}` in inline enum",
                    self.duplicate.value()
                ),
            )
        };

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.duplicate.span().from,
                self.duplicate.span(),
                "duplicate defined here",
            )
            .info_block(schema, self.first.from, self.first, "first defined here");
        }

        fmt.format()
    }
}

impl From<DuplicateEnumVariant> for Error {
    fn from(e: DuplicateEnumVariant) -> Self {
        Self::DuplicateEnumVariant(e)
    }
}
