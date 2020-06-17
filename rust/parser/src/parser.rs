use crate::issues::Issues;
use crate::validate::Validate;
use crate::{Error, Schema, Warning};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Parser {
    schema_paths: Vec<PathBuf>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            schema_paths: Vec::new(),
        }
    }

    pub fn add_schema_path<P>(&mut self, path: P)
    where
        P: Into<PathBuf>,
    {
        self.schema_paths.push(path.into());
    }

    pub fn parse<P>(&self, schema_path: P) -> Parsed
    where
        P: AsRef<Path>,
    {
        let mut issues = Issues::default();
        let main_schema = Schema::parse(schema_path, &mut issues);

        let mut parsed = Parsed {
            main_schema: main_schema.name().to_owned(),
            schemas: HashMap::new(),
            issues,
        };
        parsed
            .schemas
            .insert(main_schema.name().to_owned(), main_schema);

        parsed.validate();
        parsed
    }
}

impl Default for Parser {
    fn default() -> Self {
        Parser::new()
    }
}

#[derive(Debug)]
pub struct Parsed {
    main_schema: String,
    schemas: HashMap<String, Schema>,
    issues: Issues,
}

impl Parsed {
    fn validate(&mut self) {
        for (schema_name, schema) in &self.schemas {
            let is_main_schema = *schema_name == self.main_schema;
            let mut validate =
                Validate::new(schema_name, &mut self.issues, &self.schemas, is_main_schema);
            schema.validate(&mut validate);
        }
    }

    pub fn main_schema(&self) -> &Schema {
        self.get_schema(&self.main_schema).unwrap()
    }

    pub fn get_schema(&self, schema_name: &str) -> Option<&Schema> {
        self.schemas.get(schema_name)
    }

    pub fn errors(&self) -> &[Error] {
        self.issues.errors()
    }

    pub fn warnings(&self) -> &[Warning] {
        self.issues.warnings()
    }
}
