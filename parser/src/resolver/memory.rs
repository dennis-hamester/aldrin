use super::{Resolver, SchemaFile};
use std::collections::HashMap;
use std::io::Error;

#[derive(Debug)]
pub struct MemoryResolver {
    main_schema: Schema,
    schemas: HashMap<String, Schema>,
}

impl MemoryResolver {
    pub fn new(name: impl Into<String>, source: Result<String, Error>) -> Self {
        Self {
            main_schema: Schema::new(name.into(), source),
            schemas: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: impl Into<String>, source: Result<String, Error>) -> &mut Self {
        let name = name.into();
        self.schemas.insert(name.clone(), Schema::new(name, source));
        self
    }
}

impl Resolver for MemoryResolver {
    fn main_schema(&self) -> SchemaFile<'_> {
        self.main_schema.as_schema_file()
    }

    fn resolve(&mut self, name: &str) -> Option<SchemaFile<'_>> {
        if name == self.main_schema.name {
            Some(self.main_schema.as_schema_file())
        } else {
            self.schemas.get(name).map(Schema::as_schema_file)
        }
    }
}

#[derive(Debug)]
struct Schema {
    name: String,
    path: String,
    source: Result<String, Error>,
}

impl Schema {
    fn new(name: String, source: Result<String, Error>) -> Self {
        Self {
            path: format!("({name})"),
            name,
            source,
        }
    }

    fn as_schema_file(&self) -> SchemaFile<'_> {
        SchemaFile::new(&self.name, &self.path, self.source.as_deref())
    }
}
