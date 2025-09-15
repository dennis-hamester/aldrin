use super::{Warning, WarningKind};
use crate::ast::{ConstDef, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{util, Parser};

#[derive(Debug)]
pub(crate) struct NonShoutySnakeCaseConst {
    schema_name: String,
    shouty_snake_case: String,
    ident: Ident,
}

impl NonShoutySnakeCaseConst {
    pub(crate) fn validate(const_def: &ConstDef, validate: &mut Validate) {
        if !Ident::is_valid(const_def.name().value()) {
            return;
        }

        let shouty_snake_case = util::to_upper_case(const_def.name().value());
        if const_def.name().value() == shouty_snake_case {
            return;
        }

        validate.add_warning(Self {
            schema_name: validate.schema_name().to_owned(),
            shouty_snake_case,
            ident: const_def.name().clone(),
        });
    }
}

impl Diagnostic for NonShoutySnakeCaseConst {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.warning(format!(
            "constant `{}` should have an upper-case name",
            self.ident.value(),
        ));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
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
        Self {
            kind: WarningKind::NonShoutySnakeCaseConst(w),
        }
    }
}
