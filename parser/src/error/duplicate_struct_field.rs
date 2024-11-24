use super::Error;
use crate::ast::{Ident, StructField};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed, Span};

#[derive(Debug)]
pub struct DuplicateStructField {
    schema_name: String,
    duplicate: Ident,
    first: Span,
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
        util::find_duplicates(
            fields,
            |field| field.name().value(),
            |duplicate, first| {
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.name().clone(),
                    first: first.name().span(),
                    struct_span,
                    struct_ident: ident.cloned(),
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
            Formatter::new(
                self,
                format!(
                    "duplicate field `{}` in struct `{}`",
                    self.duplicate.value(),
                    ident.value()
                ),
            )
        } else {
            Formatter::new(
                self,
                format!(
                    "duplicate field `{}` in inline struct",
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

impl From<DuplicateStructField> for Error {
    fn from(e: DuplicateStructField) -> Self {
        Self::DuplicateStructField(e)
    }
}
