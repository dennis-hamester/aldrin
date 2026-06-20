use crate::issues::Issues;
use crate::{ParserEntry, SchemaRef};
use indexmap::IndexMap;

pub(crate) struct Validate<'a> {
    _schema_ref: SchemaRef,
    _schemas: &'a IndexMap<String, ParserEntry>,
    _is_main_schema: bool,
    _issues: &'a mut Issues,
}

impl<'a> Validate<'a> {
    pub(crate) fn new(
        schema_ref: SchemaRef,
        schemas: &'a IndexMap<String, ParserEntry>,
        is_main_schema: bool,
        issues: &'a mut Issues,
    ) -> Self {
        Self {
            _schema_ref: schema_ref,
            _schemas: schemas,
            _is_main_schema: is_main_schema,
            _issues: issues,
        }
    }

    // pub(crate) fn schema_ref(&self) -> SchemaRef {
    //     self.schema_ref
    // }

    // pub(crate) fn add_error(&mut self, e: impl Into<Error>) {
    //     self.issues.add_error(e);
    // }

    // pub(crate) fn add_warning(&mut self, w: impl Into<Warning>) {
    //     if self.is_main_schema {
    //         self.issues.add_warning(w);
    //     } else {
    //         self.issues.add_other_warning(w);
    //     }
    // }

    // pub(crate) fn schema(&self, schema_ref: SchemaRef) -> &'a ParserEntry {
    //     self.schemas.get_index(schema_ref.0).unwrap().1
    // }

    // pub(crate) fn current_schema(&self) -> &'a ParserEntry {
    //     self.schema(self.schema_ref)
    // }
}
