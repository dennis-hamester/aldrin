use super::Error;
use crate::ast::ImportStmt;
use crate::validate::Validate;

#[derive(Debug)]
pub struct ImportNotFound {
    schema_name: String,
    import: ImportStmt,
}

impl ImportNotFound {
    pub(crate) fn validate(import_stmt: &ImportStmt, validate: &mut Validate) {
        if validate
            .get_schema(import_stmt.schema_name().value())
            .is_some()
        {
            return;
        }

        validate.add_error(ImportNotFound {
            schema_name: validate.schema_name().to_owned(),
            import: import_stmt.clone(),
        });
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn import(&self) -> &ImportStmt {
        &self.import
    }
}

impl From<ImportNotFound> for Error {
    fn from(e: ImportNotFound) -> Self {
        Error::ImportNotFound(e)
    }
}
