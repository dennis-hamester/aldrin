use super::Error;
use crate::ast::{EnumVariant, Ident, LitPosInt};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed, Span};

#[derive(Debug)]
pub struct DuplicateEnumVariantId {
    schema_name: String,
    duplicate: LitPosInt,
    first: Span,
    enum_span: Span,
    enum_ident: Option<Ident>,
    free_id: u32,
}

impl DuplicateEnumVariantId {
    pub(crate) fn validate(
        vars: &[EnumVariant],
        enum_span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
        let mut max_id = vars
            .iter()
            .fold(0, |cur, var| match var.id().value().parse() {
                Ok(id) if id > cur => id,
                _ => cur,
            });

        util::find_duplicates(
            vars,
            |var| var.id().value(),
            |duplicate, first| {
                max_id += 1;
                let free_id = max_id;
                validate.add_error(Self {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.id().clone(),
                    first: first.id().span(),
                    enum_span,
                    enum_ident: ident.cloned(),
                    free_id,
                })
            },
        );
    }

    pub fn duplicate(&self) -> &LitPosInt {
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

    pub fn free_id(&self) -> u32 {
        self.free_id
    }
}

impl Diagnostic for DuplicateEnumVariantId {
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
                    "duplicate id `{}` in enum `{}`",
                    self.duplicate.value(),
                    ident.value()
                ),
            )
        } else {
            Formatter::new(
                self,
                format!("duplicate id `{}` in inline enum", self.duplicate.value()),
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

        fmt.help(format!("use a free id, e.g. {}", self.free_id));
        fmt.format()
    }
}

impl From<DuplicateEnumVariantId> for Error {
    fn from(e: DuplicateEnumVariantId) -> Self {
        Self::DuplicateEnumVariantId(e)
    }
}
