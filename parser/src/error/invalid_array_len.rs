use super::{Error, ErrorKind};
use crate::ast::{ArrayLen, ArrayLenValue, ConstValue, Ident, NamedRefKind};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub(crate) struct InvalidArrayLen {
    schema_name: String,
    len: ArrayLen,
    value: String,
    const_def: Option<(String, Ident)>,
}

impl InvalidArrayLen {
    pub(crate) fn validate(len: &ArrayLen, validate: &mut Validate) {
        let (value, const_def) = match len.value() {
            ArrayLenValue::Literal(lit) => (lit.value(), None),

            ArrayLenValue::Ref(named_ref) => {
                let (schema, ident) = match named_ref.kind() {
                    NamedRefKind::Intern(ident) => (validate.get_current_schema(), ident),

                    NamedRefKind::Extern(schema, ident) => {
                        let Some(schema) = validate.get_schema(schema.value()) else {
                            return;
                        };

                        (schema, ident)
                    }
                };

                let mut res = None;
                for def in schema.definitions() {
                    let Some(const_def) = def.as_const() else {
                        continue;
                    };

                    if const_def.name().value() != ident.value() {
                        continue;
                    }

                    if res.is_some() {
                        return;
                    }

                    match const_def.value() {
                        ConstValue::U8(lit)
                        | ConstValue::I8(lit)
                        | ConstValue::U16(lit)
                        | ConstValue::I16(lit)
                        | ConstValue::U32(lit)
                        | ConstValue::I32(lit)
                        | ConstValue::U64(lit)
                        | ConstValue::I64(lit) => {
                            res = Some((lit.value(), Some((schema.name(), const_def.name()))))
                        }

                        ConstValue::String(_) | ConstValue::Uuid(_) => return,
                    }
                }

                if let Some(res) = res {
                    res
                } else {
                    return;
                }
            }
        };

        if value.parse::<u32>().ok().map(|v| v == 0).unwrap_or(true) {
            validate.add_error(Self {
                schema_name: validate.schema_name().to_owned(),
                len: len.clone(),
                value: value.to_owned(),
                const_def: const_def.map(|(s, i)| (s.to_owned(), i.clone())),
            });
        }
    }
}

impl Diagnostic for InvalidArrayLen {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.error(format!("invalid array length {}", self.value));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.len.span(), "array length used here");
        }

        if let Some((ref schema, ref ident)) = self.const_def {
            if let Some(schema) = parsed.get_schema(schema) {
                report = report.context(schema, ident.span(), "constant defined here");
            }
        }

        report = report.help("arrays must have a length in the range from 1 to 4294967295");
        report.render()
    }
}

impl From<InvalidArrayLen> for Error {
    fn from(e: InvalidArrayLen) -> Self {
        Self {
            kind: ErrorKind::InvalidArrayLen(e),
        }
    }
}
