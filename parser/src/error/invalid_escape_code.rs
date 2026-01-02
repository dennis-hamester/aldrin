use super::{Error, ErrorKind};
use crate::ast::{ConstDef, ConstValue, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{Parser, Span};

#[derive(Debug)]
pub(crate) struct InvalidEscapeCode {
    schema_name: String,
    ident: Ident,
    escape: String,
    span: Span,
}

impl InvalidEscapeCode {
    pub(crate) fn validate(def: &ConstDef, validate: &mut Validate) {
        let ConstValue::String(val) = def.value() else {
            return;
        };

        let mut pos = val.span_inner().start;
        let mut chars = val.value_inner().chars();

        while let Some(c1) = chars.next() {
            if c1 == '\\' {
                let c2 = chars.next().unwrap();
                let len = 1 + c2.len_utf8();

                if (c2 != '\\') && (c2 != '"') {
                    validate.add_error(Self {
                        schema_name: validate.schema_name().to_owned(),
                        ident: def.name().clone(),
                        escape: format!("\\{c2}"),
                        span: Span {
                            start: pos,
                            end: pos + len,
                        },
                    });
                }

                pos += len;
            } else {
                pos += c1.len_utf8();
            }
        }
    }
}

impl Diagnostic for InvalidEscapeCode {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.error(format!(
            "invalid escape code `{}` in string constant `{}`",
            self.escape,
            self.ident.value()
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet_with_context(
                schema,
                self.span,
                format!("invalid escape code `{}`", self.escape),
                self.ident.span(),
                "string constant defined here",
            );
        }

        report = report.help("only `\\\\` and `\\\"` are supported");
        report.render()
    }
}

impl From<InvalidEscapeCode> for Error {
    fn from(e: InvalidEscapeCode) -> Self {
        Self {
            kind: ErrorKind::InvalidEscapeCode(e),
        }
    }
}
