use crate::issues::Issues;
use crate::{Error, Schema, Warning};
use std::collections::HashMap;

pub(crate) struct Validate<'a> {
    schema_name: &'a str,
    issues: &'a mut Issues,
    schemas: &'a HashMap<String, Schema>,
    is_main_schema: bool,
}

impl<'a> Validate<'a> {
    pub fn new(
        schema_name: &'a str,
        issues: &'a mut Issues,
        schemas: &'a HashMap<String, Schema>,
        is_main_schema: bool,
    ) -> Self {
        Validate {
            schema_name,
            issues,
            schemas,
            is_main_schema,
        }
    }

    pub fn schema_name(&self) -> &'a str {
        self.schema_name
    }

    pub fn add_error<E>(&mut self, e: E)
    where
        E: Into<Error>,
    {
        self.issues.add_error(e);
    }

    pub fn add_warning<W>(&mut self, w: W)
    where
        W: Into<Warning>,
    {
        self.issues.add_warning(w)
    }

    pub fn get_schema(&self, schema_name: &str) -> Option<&'a Schema> {
        self.schemas.get(schema_name)
    }

    pub fn is_main_schema(&self) -> bool {
        self.is_main_schema
    }
}
