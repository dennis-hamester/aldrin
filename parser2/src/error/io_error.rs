use crate::{DiagnosticRenderer, Parser, SchemaRef};

#[derive(Debug, Clone)]
pub(crate) struct IoError {
    schema: SchemaRef,
    err: String,
}

impl IoError {
    pub(crate) fn new(schema: SchemaRef, err: String) -> Self {
        Self { schema, err }
    }

    pub(crate) fn schema(&self) -> SchemaRef {
        self.schema
    }

    pub(crate) fn render(&self, renderer: &DiagnosticRenderer, parser: &Parser) -> String {
        let schema = parser.schema(self.schema);

        renderer
            .error(&self.err, parser)
            .note(format!("tried to read `{}`", schema.path()))
            .render()
    }
}
