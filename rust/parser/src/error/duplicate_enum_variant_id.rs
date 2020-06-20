use super::Error;
use crate::ast::{EnumVariant, Ident, LitPosInt};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::{Parsed, Span};
use std::collections::hash_map::{Entry, HashMap};

#[derive(Debug)]
pub struct DuplicateEnumVariantId {
    schema_name: String,
    duplicate: LitPosInt,
    original_span: Span,
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
        let mut ids = HashMap::new();

        let mut free_id = 1 + vars.iter().fold(0, |cur, v| match v.id().value().parse() {
            Ok(id) if id > cur => id,
            _ => cur,
        });

        for var in vars {
            match ids.entry(var.id().value()) {
                Entry::Vacant(e) => {
                    e.insert(var.id());
                }

                Entry::Occupied(e) => {
                    validate.add_error(DuplicateEnumVariantId {
                        schema_name: validate.schema_name().to_owned(),
                        duplicate: var.id().clone(),
                        original_span: e.get().span(),
                        enum_span,
                        enum_ident: ident.cloned(),
                        free_id,
                    });

                    free_id += 1;
                }
            }
        }
    }

    pub fn duplicate(&self) -> &LitPosInt {
        &self.duplicate
    }

    pub fn original_span(&self) -> Span {
        self.original_span
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
            Formatter::error(format!(
                "duplicate id `{}` in enum `{}`",
                self.duplicate.value(),
                ident.value()
            ))
        } else {
            Formatter::error(format!(
                "duplicate id `{}` in inline enum",
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

            if let Some(ref ident) = self.enum_ident {
                fmt.info_block(
                    schema,
                    ident.span().from,
                    ident.span(),
                    format!("enum `{}` defined here", ident.value()),
                );
            } else {
                fmt.info_block(
                    schema,
                    self.enum_span.from,
                    self.enum_span,
                    "inline enum defined here",
                );
            }
        }

        fmt.help(format!("use a free id like {}", self.free_id));
        fmt.format()
    }
}

impl From<DuplicateEnumVariantId> for Error {
    fn from(e: DuplicateEnumVariantId) -> Self {
        Error::DuplicateEnumVariantId(e)
    }
}
