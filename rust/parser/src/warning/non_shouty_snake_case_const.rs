use super::Warning;
use crate::ast::{ConstDef, Ident};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::validate::Validate;
use crate::Parsed;
use heck::ShoutySnakeCase;

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
            validate.add_warning(NonShoutySnakeCaseConst {
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

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(
            self,
            format!(
                "constant `{}` should have an upper-case name",
                self.ident.value()
            ),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(schema, self.ident.span().from, self.ident.span(), "");
        }

        fmt.help(format!(
            "consider renaming constant `{}` to `{}`",
            self.ident.value(),
            self.shouty_snake_case
        ));
        fmt.format()
    }
}

impl From<NonShoutySnakeCaseConst> for Warning {
    fn from(w: NonShoutySnakeCaseConst) -> Self {
        Warning::NonShoutySnakeCaseConst(w)
    }
}
