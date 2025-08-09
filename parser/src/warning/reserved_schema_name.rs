use super::Warning;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::util::{self, Language, ReservedUsage};
use crate::validate::Validate;
use crate::Parsed;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ReservedSchemaName {
    schema_name: String,
    path: PathBuf,
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

    fn format<'a>(&'a self, _parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(
            self,
            format!(
                "the schema name `{}` is reserved in some language(s)",
                self.schema_name,
            ),
        );

        fmt.note(format!(
            "the schema is located here at `{}`",
            self.path.display(),
        ));

        for (kind, langs) in self.usage {
            fmt.note(format!(
                "`{}` is {} in {}",
                self.schema_name,
                kind,
                Language::fmt_list(langs),
            ));
        }

        fmt.format()
    }
}

impl From<ReservedSchemaName> for Warning {
    fn from(w: ReservedSchemaName) -> Self {
        Self::ReservedSchemaName(w)
    }
}
