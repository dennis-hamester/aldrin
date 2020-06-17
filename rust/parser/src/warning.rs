mod non_shouty_snake_case_const;
mod non_snake_case_schema_name;

pub use non_shouty_snake_case_const::NonShoutySnakeCaseConst;
pub use non_snake_case_schema_name::NonSnakeCaseSchemaName;

#[derive(Debug)]
#[non_exhaustive]
pub enum Warning {
    NonShoutySnakeCaseConst(NonShoutySnakeCaseConst),
    NonSnakeCaseSchemaName(NonSnakeCaseSchemaName),
}

impl Warning {
    pub fn schema_name(&self) -> &str {
        match self {
            Warning::NonShoutySnakeCaseConst(w) => w.schema_name(),
            Warning::NonSnakeCaseSchemaName(w) => w.schema_name(),
        }
    }
}
