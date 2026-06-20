use super::{Resolver, SchemaFile};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MemoryResolver {
    main_schema: Option<(String, Result<String, String>)>,
    schemas: HashMap<String, Result<String, String>>,
}

impl MemoryResolver {
    pub fn new(name: impl Into<String>, source: Result<String, String>) -> Self {
        Self {
            main_schema: Some((name.into(), source)),
            schemas: HashMap::new(),
        }
    }

    pub fn with_source(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self::new(name, Ok(source.into()))
    }

    pub fn with_error(name: impl Into<String>, err: &impl ToString) -> Self {
        Self::new(name, Err(err.to_string()))
    }

    pub fn add(&mut self, name: impl Into<String>, source: Result<String, String>) -> &mut Self {
        self.schemas.insert(name.into(), source);
        self
    }

    pub fn add_with_source(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> &mut Self {
        self.add(name, Ok(source.into()))
    }

    pub fn add_with_error(&mut self, name: impl Into<String>, err: &impl ToString) -> &mut Self {
        self.add(name, Err(err.to_string()))
    }
}

impl Resolver for MemoryResolver {
    fn main_schema(&mut self) -> (Cow<'_, str>, SchemaFile) {
        let (name, source) = self.main_schema.take().expect("main schema already taken");
        let path = format!("({name})");
        (Cow::Owned(name), SchemaFile::new(path, source))
    }

    fn resolve(&mut self, name: &str) -> Option<SchemaFile> {
        self.schemas
            .remove(name)
            .map(|source| SchemaFile::new(format!("({name})"), source))
    }
}
