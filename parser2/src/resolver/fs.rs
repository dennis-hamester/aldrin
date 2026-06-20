use super::{Resolver, SchemaFile};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FilesystemResolver {
    main_schema: PathBuf,
    include_paths: Vec<PathBuf>,
}

impl FilesystemResolver {
    pub fn new(main_schema: impl Into<PathBuf>) -> Self {
        Self {
            main_schema: main_schema.into(),
            include_paths: Vec::new(),
        }
    }

    pub fn with_include_paths(
        main_schema: impl Into<PathBuf>,
        include_paths: impl IntoIterator<Item: Into<PathBuf>>,
    ) -> Self {
        Self {
            main_schema: main_schema.into(),
            include_paths: include_paths.into_iter().map(Into::into).collect(),
        }
    }

    pub fn add_include_path(&mut self, include_path: impl Into<PathBuf>) -> &mut Self {
        self.include_paths.push(include_path.into());
        self
    }

    pub fn add_include_paths<T>(
        &mut self,
        include_paths: impl IntoIterator<Item: Into<PathBuf>>,
    ) -> &mut Self {
        self.include_paths
            .extend(include_paths.into_iter().map(Into::into));
        self
    }
}

impl Resolver for FilesystemResolver {
    fn main_schema(&mut self) -> (Cow<'_, str>, SchemaFile) {
        let path = self.main_schema.to_string_lossy().into_owned();

        let name = match self.main_schema.file_stem().map(OsStr::to_str) {
            Some(Some(name)) => name,

            Some(None) => {
                return (
                    Cow::Borrowed(""),
                    SchemaFile::with_error(path, &"non UTF-8 file stem"),
                );
            }

            None => "",
        };

        let source = fs::read_to_string(&self.main_schema).map_err(|e| e.to_string());
        (Cow::Borrowed(name), SchemaFile::new(path, source))
    }

    fn resolve(&mut self, name: &str) -> Option<SchemaFile> {
        for path in self.include_paths.iter().rev() {
            let mut path = path.join(name);
            path.set_extension("aldrin");

            match fs::read_to_string(&path) {
                Ok(source) => {
                    let path = path
                        .into_os_string()
                        .into_string()
                        .unwrap_or_else(|path| path.to_string_lossy().into_owned());

                    return Some(SchemaFile::with_source(path, source));
                }

                Err(e) if e.kind() == ErrorKind::NotFound => {}

                Err(e) => {
                    let path = path
                        .into_os_string()
                        .into_string()
                        .unwrap_or_else(|path| path.to_string_lossy().into_owned());

                    return Some(SchemaFile::with_error(path, &e));
                }
            }
        }

        None
    }
}
