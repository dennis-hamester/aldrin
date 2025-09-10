use super::Warning;
use crate::ast::{ConstDef, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::Parsed;
use heck::ToShoutySnakeCase;

#[derive(Debug)]
pub struct NonShoutySnakeCaseConst {
    schema_name: String,
    shouty_snake_case: String,
    ident: Ident,
}

impl NonShoutySnakeCaseConst {
    pub(crate) fn validate(const_def: &ConstDef, validate: &mut Validate) {
        let shouty_snake_case = const_def.name().value().to_shouty_snake_case();
        if const_def.name().value() != shouty_snake_case {
            validate.add_warning(Self {
                schema_name: validate.schema_name().to_owned(),
                shouty_snake_case,
                ident: const_def.name().clone(),
            });
        }
    }

    pub fn shouty_snake_case(&self) -> &str {
        &self.shouty_snake_case
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Diagnostic for NonShoutySnakeCaseConst {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.warning(format!(
            "constant `{}` should have an upper-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ident.span(), "");
        }

        report = report.help(format!(
            "consider renaming constant `{}` to `{}`",
            self.ident.value(),
            self.shouty_snake_case
        ));

        report.render()
    }
}

impl From<NonShoutySnakeCaseConst> for Warning {
    fn from(w: NonShoutySnakeCaseConst) -> Self {
        Self::NonShoutySnakeCaseConst(w)
    }
}
