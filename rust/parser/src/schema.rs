use crate::ast::SchemaName;
use crate::error::{InvalidSchemaName, IoError, ParserError};
use crate::grammar::{Grammar, Rule};
use crate::issues::Issues;
use pest::Parser;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Schema {
    name: String,
    path: PathBuf,
    source: String,
}

impl Schema {
    pub(crate) fn parse<P>(schema_path: P, issues: &mut Issues) -> Option<Self>
    where
        P: AsRef<Path>,
    {
        let schema_path = schema_path.as_ref();
        let mut file = match File::open(schema_path) {
            Ok(file) => file,
            Err(e) => {
                issues.add_error(IoError::new(schema_path, e));
                return None;
            }
        };

        let mut source = String::new();
        if let Err(e) = file.read_to_string(&mut source) {
            issues.add_error(IoError::new(schema_path, e));
            return None;
        }

        let mut schema = Schema {
            name: Schema::parse_file_name(schema_path, issues),
            path: schema_path.to_owned(),
            source,
        };

        let pairs = match Grammar::parse(Rule::file, &schema.source) {
            Ok(pairs) => pairs,
            Err(e) => {
                issues.add_error(ParserError::new(&schema.name, e));
                return Some(schema);
            }
        };

        Some(schema)
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

        let schema_name = SchemaName::parse(schema_name_pairs.next().unwrap(), issues, true);
        if schema_name.span().to.index != file_stem_str.len() {
            issues.add_error(InvalidSchemaName::new(file_stem_str));
        }

        schema_name.value().to_owned()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}
