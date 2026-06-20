mod fs;
mod memory;

use std::borrow::Cow;

pub use fs::FilesystemResolver;
pub use memory::MemoryResolver;

pub trait Resolver {
    fn main_schema(&mut self) -> (Cow<'_, str>, SchemaFile);
    fn resolve(&mut self, name: &str) -> Option<SchemaFile>;
}

impl<T: Resolver + ?Sized> Resolver for &mut T {
    fn main_schema(&mut self) -> (Cow<'_, str>, SchemaFile) {
        (**self).main_schema()
    }

    fn resolve(&mut self, name: &str) -> Option<SchemaFile> {
        (**self).resolve(name)
    }
}

impl<T: Resolver + ?Sized> Resolver for Box<T> {
    fn main_schema(&mut self) -> (Cow<'_, str>, SchemaFile) {
        (**self).main_schema()
    }

    fn resolve(&mut self, name: &str) -> Option<SchemaFile> {
        (**self).resolve(name)
    }
}

#[derive(Debug)]
pub struct SchemaFile {
    pub(crate) path: String,
    pub(crate) source: Result<String, String>,
}

impl SchemaFile {
    pub fn new(path: impl Into<String>, source: Result<String, String>) -> Self {
        Self {
            path: path.into(),
            source,
        }
    }

    pub fn with_source(path: impl Into<String>, source: impl Into<String>) -> Self {
        Self::new(path, Ok(source.into()))
    }

    pub fn with_error(path: impl Into<String>, err: &impl ToString) -> Self {
        Self::new(path, Err(err.to_string()))
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn source(&self) -> Result<&str, &str> {
        match self.source {
            Ok(ref src) => Ok(src),
            Err(ref e) => Err(e),
        }
    }
}
