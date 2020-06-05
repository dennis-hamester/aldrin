use crate::issues::Issues;
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
        let mut parsed = Parsed {
            main_schema: None,
            schemas: HashMap::new(),
            issues: Issues::default(),
        };

        match Schema::parse(schema_path, &mut parsed.issues) {
            Some(schema) => {
                let main_schema = schema.name().to_owned();
                parsed.main_schema = Some(main_schema.clone());
                parsed.schemas.insert(main_schema, schema);
            }

            None => return parsed,
        }

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
    main_schema: Option<String>,
    schemas: HashMap<String, Schema>,
    issues: Issues,
}

impl Parsed {
    pub fn main_schema(&self) -> Option<&str> {
        self.main_schema.as_deref()
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
