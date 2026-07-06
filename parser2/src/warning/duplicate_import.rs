use crate::ast::Ident;
use crate::validate::Validate;
use crate::{DiagnosticRenderer, Parser, SchemaRef, Spanned};
use crate::{Schema, util};

#[derive(Debug, Copy, Clone)]
pub(crate) struct DuplicateImport {
    schema: SchemaRef,
    duplicate: Ident,
    first: Ident,
}

impl DuplicateImport {
    pub(crate) fn validate(schema: &Schema, validate: &mut Validate) {
        let src = validate.current_source();

        util::find_duplicates(
            &schema.imports,
            |import| import.schema.source(src),
            |duplicate, first| {
                validate.add_warning(Self {
                    schema: validate.current_schema_ref(),
                    duplicate: duplicate.schema,
                    first: first.schema,
                });
            },
        );
    }

    pub(crate) fn schema(&self) -> SchemaRef {
        self.schema
    }

    pub(crate) fn render(&self, renderer: &DiagnosticRenderer, parser: &Parser) -> String {
        let schema = parser.schema(self.schema);
        let import = schema.source_of(&self.duplicate).unwrap();

        renderer
            .warning(format!("duplicate import of schema `{import}`"), parser)
            .snippet(self.schema, &self.duplicate, "duplicate import")
            .context(self.schema, &self.first, "first imported here")
            .help("remove the duplicate import statement")
            .render()
    }
}
