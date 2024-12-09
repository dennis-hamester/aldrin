use super::Error;
use crate::ast::Ident;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;

const RESERVED: &[&str] = &[
    "bool",
    "box",
    "bytes",
    "const",
    "enum",
    "event",
    "f32",
    "f64",
    "fn",
    "i16",
    "i32",
    "i64",
    "i8",
    "import",
    "lifetime",
    "map",
    "object_id",
    "option",
    "receiver",
    "required",
    "result",
    "sender",
    "service",
    "service_id",
    "set",
    "string",
    "struct",
    "u16",
    "u32",
    "u64",
    "u8",
    "unit",
    "uuid",
    "value",
    "vec",
];

#[derive(Debug)]
pub struct ExpectedIdentFoundReserved {
    schema_name: String,
    ident: Ident,
}

impl ExpectedIdentFoundReserved {
    pub(crate) fn validate(ident: &Ident, validate: &mut Validate) {
        if RESERVED.contains(&ident.value()) {
            validate.add_error(Self {
                schema_name: validate.schema_name().to_owned(),
                ident: ident.clone(),
            });
        }
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Diagnostic for ExpectedIdentFoundReserved {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(
            self,
            format!(
                "expected identifer; found reserved name `{}`",
                self.ident.value()
            ),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.ident.span().from,
                self.ident.span(),
                "identifier expected here",
            );
        }

        fmt.note(format!(
            "the name `{}` is reserved and cannot be used as an identifer",
            self.ident.value()
        ));

        fmt.format()
    }
}

impl From<ExpectedIdentFoundReserved> for Error {
    fn from(e: ExpectedIdentFoundReserved) -> Self {
        Self::ExpectedIdentFoundReserved(e)
    }
}
