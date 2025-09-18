use crate::ast::{Definition, Ident, ImportStmt, Prelude};
use crate::error::{DuplicateDefinition, InvalidSchemaName, InvalidSyntax, IoError};
use crate::grammar::{Grammar, Rule};
use crate::issues::Issues;
use crate::validate::Validate;
use crate::warning::{DuplicateImport, NonSnakeCaseSchemaName, ReservedSchemaName};
use crate::SchemaFile;
use pest::Parser;

#[derive(Debug, Clone)]
pub struct Schema {
    name: String,
    path: String,
    source: Option<String>,
    doc: Option<String>,
    imports: Vec<ImportStmt>,
    defs: Vec<Definition>,
}

impl Schema {
    pub(crate) fn parse(file: &SchemaFile, issues: &mut Issues) -> Self {
        let mut schema = Self {
            name: file.name().to_owned(),
            path: file.path().to_owned(),
            source: None,
            doc: None,
            imports: Vec::new(),
            defs: Vec::new(),
        };

        let source = match file.source() {
            Ok(source) => source,

            Err(e) => {
                issues.add_error(IoError::new(&schema.name, e.to_string()));
                return schema;
            }
        };

        schema.source = Some(source.to_owned());

        let mut pairs = match Grammar::parse(Rule::file, source) {
            Ok(pairs) => pairs,

            Err(e) => {
                issues.add_error(InvalidSyntax::new(&schema.name, e));
                return schema;
            }
        };

        let mut prelude = Prelude::new(&mut pairs, true);

        for pair in pairs {
            match pair.as_rule() {
                Rule::import_stmt => schema.imports.push(ImportStmt::parse(pair)),
                Rule::def => schema.defs.push(Definition::parse(pair)),
                Rule::EOI => break,
                _ => unreachable!(),
            }
        }

        schema.doc = prelude.take_inline_doc().into();

        schema
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        if !Ident::is_valid(&self.name) {
            validate.add_error(InvalidSchemaName::new(self.name.clone()));
        }

        if self.source.is_none() {
            return;
        }

        DuplicateDefinition::validate(self, validate);
        DuplicateImport::validate(self, validate);
        NonSnakeCaseSchemaName::validate(&self.name, validate);
        ReservedSchemaName::validate(&self.name, validate);

        for import in &self.imports {
            import.validate(validate);
        }

        for def in &self.defs {
            def.validate(validate);
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn imports(&self) -> &[ImportStmt] {
        &self.imports
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.defs
    }
}
