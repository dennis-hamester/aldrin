use super::{Warning, WarningKind};
use crate::Parser;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::util::{self, Language, ReservedUsage};
use crate::validate::Validate;

#[derive(Debug)]
pub(crate) struct ReservedSchemaName {
    schema_name: String,
    path: String,
    usage: ReservedUsage,
}

impl ReservedSchemaName {
    pub(crate) fn validate(schema_name: &str, validate: &mut Validate) {
        if let Some(usage) = util::is_reserved_name(schema_name) {
            validate.add_warning(Self {
                schema_name: schema_name.to_owned(),
                path: validate.get_current_schema().path().to_owned(),
                usage,
            })
        }
    }
}

impl Diagnostic for ReservedSchemaName {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, _parser: &Parser) -> String {
        let mut report = renderer.warning(format!(
            "the schema name `{}` is reserved in some language(s)",
            self.schema_name,
        ));

        report = report.note(format!("the schema is located at `{}`", self.path));

        for (kind, langs) in self.usage {
            report = report.note(format!(
                "`{}` is {} in {}",
                self.schema_name,
                kind,
                Language::fmt_list(langs),
            ));
        }

        report.render()
    }
}

impl From<ReservedSchemaName> for Warning {
    fn from(w: ReservedSchemaName) -> Self {
        Self {
            kind: WarningKind::ReservedSchemaName(w),
        }
    }
}
