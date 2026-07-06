use crate::ast::{Ident, Import};
use crate::validate::Validate;
use crate::{DiagnosticRenderer, Parser, SchemaRef, Spanned};

#[derive(Debug, Copy, Clone)]
pub(crate) struct ImportNotFound {
    schema: SchemaRef,
    import: Ident,
}

impl ImportNotFound {
    pub(crate) fn validate(import: &Import, validate: &mut Validate) {
        let entry = validate.current_entry();
        let source = entry.source().unwrap();
        let name = import.schema.source(source);

        if validate.entry_by_name(name).is_none() {
            validate.add_error(Self {
                schema: entry.schema_ref(),
                import: import.schema,
            });
        }
    }

    pub(crate) fn schema(&self) -> SchemaRef {
        self.schema
    }

    pub(crate) fn render(&self, renderer: &DiagnosticRenderer, parser: &Parser) -> String {
        let schema = parser.schema(self.schema);
        let import = schema.source_of(&self.import).unwrap();

        renderer
            .error(format!("schema `{import}` not found"), parser)
            .snippet(self.schema, &self.import, "")
            .help("an include directory may be missing or incorrect")
            .render()
    }
}
