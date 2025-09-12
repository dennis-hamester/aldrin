use super::{Resolver, SchemaFile};
use std::borrow::Cow;
use std::collections::hash_map::{Entry, HashMap};
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct FilesystemResolver {
    include_paths: Vec<PathBuf>,
    main_schema: Schema,
    schemas: HashMap<String, Schema>,
}

impl FilesystemResolver {
    pub fn new(main_schema: impl AsRef<Path>) -> Self {
        Self {
            include_paths: Vec::new(),
            main_schema: Schema::open(main_schema.as_ref()),
            schemas: HashMap::new(),
        }
    }

    pub fn with_include_paths<T>(main_schema: impl AsRef<Path>, include_paths: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<PathBuf>,
    {
        let mut this = Self::new(main_schema);
        this.add_include_paths(include_paths);
        this
    }

    pub fn add_include_path(&mut self, include_path: impl Into<PathBuf>) -> &mut Self {
        self.include_paths.push(include_path.into());
        self
    }

    pub fn add_include_paths<T>(&mut self, include_paths: T) -> &mut Self
    where
        T: IntoIterator,
        T::Item: Into<PathBuf>,
    {
        self.include_paths
            .extend(include_paths.into_iter().map(Into::into));
        self
    }
}

impl Resolver for FilesystemResolver {
    fn main_schema(&self) -> SchemaFile<'_> {
        self.main_schema.as_schema_file()
    }

    fn resolve(&mut self, name: &str) -> Option<SchemaFile<'_>> {
        if name == self.main_schema.name {
            return Some(self.main_schema.as_schema_file());
        }

        let entry = match self.schemas.entry(name.to_owned()) {
            Entry::Occupied(entry) => return Some(entry.into_mut().as_schema_file()),
            Entry::Vacant(entry) => entry,
        };

        let path = self
            .include_paths
            .iter()
            .rev()
            .cloned()
            .find_map(|mut path| {
                path.push(name);
                path.set_extension("aldrin");

                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            })?;

        let schema = Schema::open_with_name(name.to_owned(), &path);
        let schema = entry.insert(schema);
        Some(schema.as_schema_file())
    }
}

#[derive(Debug)]
struct Schema {
    name: String,
    path: String,
    source: Result<String, Error>,
}

impl Schema {
    fn open(path: &Path) -> Self {
        let name = Self::schema_name_from_path(path).into_owned();
        Self::open_with_name(name, path)
    }

    fn open_with_name(name: String, path: &Path) -> Self {
        Self {
            name,
            path: path.to_string_lossy().into_owned(),
            source: fs::read_to_string(path),
        }
    }

    fn schema_name_from_path(path: &Path) -> Cow<'_, str> {
        match path.file_stem() {
            Some(stem) => stem.to_string_lossy(),
            None => Cow::Borrowed(""),
        }
    }

    fn as_schema_file(&self) -> SchemaFile<'_> {
        SchemaFile::new(&self.name, &self.path, self.source.as_deref())
    }
}
