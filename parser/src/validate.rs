use crate::issues::Issues;
use crate::{Error, Schema, Warning};
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) struct Validate<'a> {
    schema_name: &'a str,
    issues: &'a mut Issues,
    schemas: &'a HashMap<String, Schema>,
    is_main_schema: bool,
    schema_paths: &'a [PathBuf],
}

impl<'a> Validate<'a> {
    pub(crate) fn new(
        schema_name: &'a str,
        issues: &'a mut Issues,
        schemas: &'a HashMap<String, Schema>,
        is_main_schema: bool,
        schema_paths: &'a [PathBuf],
    ) -> Self {
        Validate {
            schema_name,
            issues,
            schemas,
            is_main_schema,
            schema_paths,
        }
    }

    pub(crate) fn schema_name(&self) -> &'a str {
        self.schema_name
    }

    pub(crate) fn add_error<E>(&mut self, e: E)
    where
        E: Into<Error>,
    {
        self.issues.add_error(e);
    }

    pub(crate) fn add_warning<W>(&mut self, w: W)
    where
        W: Into<Warning>,
    {
        if self.is_main_schema {
            self.issues.add_warning(w);
        } else {
            self.issues.add_other_warning(w);
        }
    }

    pub(crate) fn get_schema(&self, schema_name: &str) -> Option<&'a Schema> {
        self.schemas.get(schema_name)
    }

    pub(crate) fn get_current_schema(&self) -> &'a Schema {
        self.get_schema(self.schema_name).unwrap()
    }

    pub(crate) fn schema_paths(&self) -> &'a [PathBuf] {
        self.schema_paths
    }
}
