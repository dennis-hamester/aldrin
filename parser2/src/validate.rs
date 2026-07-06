use crate::ast::{Import, Schema};
use crate::error::ImportNotFound;
use crate::issues::Issues;
use crate::warning::{DuplicateImport, UnusedImport};
use crate::{Error, ParserEntry, SchemaRef, Visitor, Warning};
use indexmap::IndexMap;
use std::ops::ControlFlow;

pub(crate) struct Validate<'a> {
    schema_ref: SchemaRef,
    schemas: &'a IndexMap<String, ParserEntry>,
    is_main_schema: bool,
    issues: &'a mut Issues,
}

impl<'a> Validate<'a> {
    pub(crate) fn run(
        schema_ref: SchemaRef,
        schemas: &'a IndexMap<String, ParserEntry>,
        is_main_schema: bool,
        issues: &'a mut Issues,
    ) {
        let mut this = Self::new(schema_ref, schemas, is_main_schema, issues);
        let schema = this.current_schema();

        schema.visit(ValidateVisitor(&mut this));
    }

    fn new(
        schema_ref: SchemaRef,
        schemas: &'a IndexMap<String, ParserEntry>,
        is_main_schema: bool,
        issues: &'a mut Issues,
    ) -> Self {
        Self {
            schema_ref,
            schemas,
            is_main_schema,
            issues,
        }
    }

    pub(crate) fn add_error(&mut self, e: impl Into<Error>) {
        self.issues.add_error(e);
    }

    pub(crate) fn add_warning(&mut self, w: impl Into<Warning>) {
        if self.is_main_schema {
            self.issues.add_warning(w);
        } else {
            self.issues.add_other_warning(w);
        }
    }

    pub(crate) fn entry(&self, schema_ref: SchemaRef) -> &'a ParserEntry {
        self.schemas.get_index(schema_ref.0).unwrap().1
    }

    pub(crate) fn entry_by_name(&self, name: &str) -> Option<&'a ParserEntry> {
        self.schemas.get(name)
    }

    pub(crate) fn current_schema_ref(&self) -> SchemaRef {
        self.schema_ref
    }

    pub(crate) fn current_entry(&self) -> &'a ParserEntry {
        self.entry(self.schema_ref)
    }

    pub(crate) fn current_schema(&self) -> &'a Schema {
        self.current_entry().schema().unwrap()
    }

    pub(crate) fn current_source(&self) -> &'a str {
        self.current_entry().source().unwrap()
    }
}

struct ValidateVisitor<'a>(&'a mut Validate<'a>);

impl<'a> Visitor<'a> for ValidateVisitor<'a> {
    type Output = ();

    fn schema(&mut self, schema: &Schema) -> ControlFlow<()> {
        DuplicateImport::validate(schema, self.0);

        ControlFlow::Continue(())
    }

    fn import(&mut self, _schema: &Schema, import: &Import) -> ControlFlow<()> {
        ImportNotFound::validate(import, self.0);
        UnusedImport::validate(import, self.0);

        ControlFlow::Continue(())
    }
}
