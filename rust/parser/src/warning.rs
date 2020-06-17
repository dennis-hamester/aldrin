mod duplicate_import;
mod non_shouty_snake_case_const;
mod non_snake_case_schema_name;

pub use duplicate_import::DuplicateImport;
pub use non_shouty_snake_case_const::NonShoutySnakeCaseConst;
pub use non_snake_case_schema_name::NonSnakeCaseSchemaName;

#[derive(Debug)]
#[non_exhaustive]
pub enum Warning {
    DuplicateImport(DuplicateImport),
    NonShoutySnakeCaseConst(NonShoutySnakeCaseConst),
    NonSnakeCaseSchemaName(NonSnakeCaseSchemaName),
}

impl Warning {
    pub fn schema_name(&self) -> &str {
        match self {
            Warning::DuplicateImport(w) => w.schema_name(),
            Warning::NonShoutySnakeCaseConst(w) => w.schema_name(),
            Warning::NonSnakeCaseSchemaName(w) => w.schema_name(),
        }
    }
}
