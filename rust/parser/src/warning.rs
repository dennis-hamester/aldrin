mod non_snake_case_schema_name;

pub use non_snake_case_schema_name::NonSnakeCaseSchemaName;

#[derive(Debug)]
#[non_exhaustive]
pub enum Warning {
    NonSnakeCaseSchemaName(NonSnakeCaseSchemaName),
}

impl Warning {
    pub fn schema_name(&self) -> &str {
        match self {
            Warning::NonSnakeCaseSchemaName(w) => w.schema_name(),
        }
    }
}
