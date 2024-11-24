use crate::error::DuplicateServiceUuid;
use crate::issues::Issues;
use crate::validate::Validate;
use crate::{Error, Schema, Warning};
use std::collections::hash_map::{Entry, HashMap};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Parser {
    schema_paths: Vec<PathBuf>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
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

        let mut imports = parsed
            .main_schema()
            .imports()
            .iter()
            .map(|i| i.schema_name().value().to_owned())
            .collect::<Vec<_>>();
        while let Some(import) = imports.pop() {
            let entry = match parsed.schemas.entry(import) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(entry) => entry,
            };

            let schema_path = match self.find_schema(entry.key()) {
                Some(schema_path) => schema_path,
                None => continue,
            };

            let schema = Schema::parse(schema_path, &mut parsed.issues);
            imports.extend(
                schema
                    .imports()
                    .iter()
                    .map(|i| i.schema_name().value().to_owned()),
            );
            entry.insert(schema);
        }

        parsed.validate(&self.schema_paths);
        parsed
    }

    fn find_schema(&self, schema_name: &str) -> Option<PathBuf> {
        for mut path in self.schema_paths.iter().rev().cloned() {
            path.push(schema_name);
            path.set_extension("aldrin");

            if path.is_file() {
                return Some(path);
            }
        }

        None
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Parsed {
    main_schema: String,
    schemas: HashMap<String, Schema>,
    issues: Issues,
}

impl Parsed {
    fn validate(&mut self, schema_paths: &[PathBuf]) {
        DuplicateServiceUuid::validate(self.schemas.values(), &mut self.issues);

        for (schema_name, schema) in &self.schemas {
            let is_main_schema = *schema_name == self.main_schema;
            let mut validate = Validate::new(
                schema_name,
                &mut self.issues,
                &self.schemas,
                is_main_schema,
                schema_paths,
            );
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

    pub fn other_warnings(&self) -> &[Warning] {
        self.issues.other_warnings()
    }
}
