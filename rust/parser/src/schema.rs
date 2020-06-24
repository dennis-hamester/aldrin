use crate::ast::{Definition, ImportStmt, SchemaName};
use crate::error::{DuplicateDefinition, InvalidSchemaName, InvalidSyntax, IoError};
use crate::grammar::{Grammar, Rule};
use crate::issues::Issues;
use crate::validate::Validate;
use crate::warning::{DuplicateImport, NonSnakeCaseSchemaName};
use pest::Parser;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Schema {
    name: String,
    path: PathBuf,
    source: Option<String>,
    imports: Vec<ImportStmt>,
    defs: Vec<Definition>,
}

impl Schema {
    pub(crate) fn parse<P>(schema_path: P, issues: &mut Issues) -> Self
    where
        P: AsRef<Path>,
    {
        let schema_path = schema_path.as_ref();

        let mut schema = Schema {
            name: Schema::parse_file_name(schema_path, issues),
            path: schema_path.to_owned(),
            source: None,
            imports: Vec::new(),
            defs: Vec::new(),
        };

        let source = {
            let mut file = match File::open(schema_path) {
                Ok(file) => file,
                Err(e) => {
                    issues.add_error(IoError::new(&schema.name, e));
                    return schema;
                }
            };

            let mut source = String::new();
            if let Err(e) = file.read_to_string(&mut source) {
                issues.add_error(IoError::new(&schema.name, e));
                return schema;
            }

            source
        };

        let pairs = match Grammar::parse(Rule::file, &source) {
            Ok(pairs) => pairs,
            Err(e) => {
                schema.source = Some(source);
                issues.add_error(InvalidSyntax::new(&schema.name, e));
                return schema;
            }
        };

        for pair in pairs {
            match pair.as_rule() {
                Rule::import_stmt => schema.imports.push(ImportStmt::parse(pair)),
                Rule::def => schema.defs.push(Definition::parse(pair)),
                Rule::EOI => break,
                _ => unreachable!(),
            }
        }

        schema.source = Some(source);
        schema
    }

    fn parse_file_name<P>(path: P, issues: &mut Issues) -> String
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        let file_stem = match path.file_stem() {
            Some(file_stem) => file_stem,
            None => {
                let schema_name = path.to_string_lossy().into_owned();
                issues.add_error(InvalidSchemaName::new(&schema_name));
                return schema_name;
            }
        };

        let file_stem_str = match file_stem.to_str() {
            Some(file_stem_str) => file_stem_str,
            None => {
                let schema_name = file_stem.to_string_lossy().into_owned();
                issues.add_error(InvalidSchemaName::new(&schema_name));
                return schema_name;
            }
        };

        let mut schema_name_pairs = match Grammar::parse(Rule::schema_name, &file_stem_str) {
            Ok(schema_name_pairs) => schema_name_pairs,
            Err(_) => {
                issues.add_error(InvalidSchemaName::new(file_stem_str));
                return file_stem_str.to_owned();
            }
        };

        if schema_name_pairs.as_str() != file_stem_str {
            issues.add_error(InvalidSchemaName::new(file_stem_str));
            return file_stem_str.to_owned();
        }

        let schema_name = SchemaName::parse(schema_name_pairs.next().unwrap());
        schema_name.value().to_owned()
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        if self.source.is_none() {
            return;
        }

        DuplicateDefinition::validate(self, validate);
        NonSnakeCaseSchemaName::validate(&self.name, validate);
        DuplicateImport::validate(self, validate);

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

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn imports(&self) -> &[ImportStmt] {
        &self.imports
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.defs
    }
}
