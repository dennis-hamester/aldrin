use super::{Warning, WarningKind};
use crate::ast::Ident;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::util::{self, Language, ReservedUsage};
use crate::validate::Validate;
use crate::Parsed;

#[derive(Debug)]
pub(crate) struct ReservedIdent {
    schema_name: String,
    ident: Ident,
    usage: ReservedUsage,
}

impl ReservedIdent {
    pub(crate) fn validate(ident: &Ident, validate: &mut Validate) {
        if let Some(usage) = util::is_reserved_name(ident.value()) {
            validate.add_warning(Self {
                schema_name: validate.schema_name().to_owned(),
                ident: ident.clone(),
                usage,
            })
        }
    }
}

impl Diagnostic for ReservedIdent {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut report = renderer.warning(format!(
            "the identifer `{}` is reserved in some language(s)",
            self.ident.value(),
        ));

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.ident.span(), "keyword used here");
        }

        for (kind, langs) in self.usage {
            report = report.note(format!(
                "`{}` is {} in {}",
                self.ident.value(),
                kind,
                Language::fmt_list(langs),
            ));
        }

        report.render()
    }
}

impl From<ReservedIdent> for Warning {
    fn from(w: ReservedIdent) -> Self {
        Self {
            kind: WarningKind::ReservedIdent(w),
        }
    }
}
