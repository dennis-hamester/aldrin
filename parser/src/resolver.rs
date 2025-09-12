mod fs;
mod memory;

use std::io::Error;

pub use fs::FilesystemResolver;
pub use memory::MemoryResolver;

pub trait Resolver {
    fn main_schema(&self) -> SchemaFile<'_>;
    fn resolve(&mut self, name: &str) -> Option<SchemaFile<'_>>;
}

impl<T: Resolver + ?Sized> Resolver for &mut T {
    fn main_schema(&self) -> SchemaFile<'_> {
        (**self).main_schema()
    }

    fn resolve(&mut self, name: &str) -> Option<SchemaFile<'_>> {
        (**self).resolve(name)
    }
}

impl<T: Resolver + ?Sized> Resolver for Box<T> {
    fn main_schema(&self) -> SchemaFile<'_> {
        (**self).main_schema()
    }

    fn resolve(&mut self, name: &str) -> Option<SchemaFile<'_>> {
        (**self).resolve(name)
    }
}

#[derive(Debug)]
pub struct SchemaFile<'a> {
    name: &'a str,
    path: &'a str,
    source: Result<&'a str, &'a Error>,
}

impl<'a> SchemaFile<'a> {
    pub fn new<N, P, S>(name: &'a N, path: &'a P, source: Result<&'a S, &'a Error>) -> Self
    where
        N: AsRef<str> + ?Sized,
        P: AsRef<str> + ?Sized,
        S: AsRef<str> + ?Sized,
    {
        Self {
            name: name.as_ref(),
            path: path.as_ref(),
            source: source.map(AsRef::as_ref),
        }
    }

    pub fn with_source<N, P, S>(name: &'a N, path: &'a P, source: &'a S) -> Self
    where
        N: AsRef<str> + ?Sized,
        P: AsRef<str> + ?Sized,
        S: AsRef<str> + ?Sized,
    {
        Self::new(name, path, Ok(source))
    }

    pub fn with_error<N, P>(name: &'a N, path: &'a P, err: &'a Error) -> Self
    where
        N: AsRef<str> + ?Sized,
        P: AsRef<str> + ?Sized,
    {
        Self::new(name, path, Err::<&str, _>(err))
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn path(&self) -> &'a str {
        self.path
    }

    pub fn source(&self) -> Result<&'a str, &'a Error> {
        self.source
    }
}
