use super::Error;
use crate::ast::{Ident, LitPosInt, StructField};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{util, Parsed, Span};

#[derive(Debug)]
pub struct DuplicateStructFieldId {
    schema_name: String,
    duplicate: LitPosInt,
    first: Span,
    struct_span: Span,
    struct_ident: Option<Ident>,
    free_id: u32,
}

impl DuplicateStructFieldId {
    pub(crate) fn validate(
        fields: &[StructField],
        struct_span: Span,
        ident: Option<&Ident>,
        validate: &mut Validate,
    ) {
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
                validate.add_error(DuplicateStructFieldId {
                    schema_name: validate.schema_name().to_owned(),
                    duplicate: duplicate.id().clone(),
                    first: first.id().span(),
                    struct_span,
                    struct_ident: ident.cloned(),
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

    pub fn struct_span(&self) -> Span {
        self.struct_span
    }

    pub fn struct_ident(&self) -> Option<&Ident> {
        self.struct_ident.as_ref()
    }

    pub fn free_id(&self) -> u32 {
        self.free_id
    }
}

impl Diagnostic for DuplicateStructFieldId {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = if let Some(ref ident) = self.struct_ident {
            Formatter::error(format!(
                "duplicate id `{}` in struct `{}`",
                self.duplicate.value(),
                ident.value()
            ))
        } else {
            Formatter::error(format!(
                "duplicate id `{}` in inline struct",
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
            .info_block(schema, self.first.from, self.first, "first defined here");
        }

        fmt.help(format!("use a free id, e.g. {}", self.free_id));
        fmt.format()
    }
}

impl From<DuplicateStructFieldId> for Error {
    fn from(e: DuplicateStructFieldId) -> Self {
        Error::DuplicateStructFieldId(e)
    }
}
