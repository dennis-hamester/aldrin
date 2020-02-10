use pest::error::Error as PestError;
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::path::PathBuf;

#[derive(Debug)]
#[non_exhaustive]
pub struct Error {
    pub kind: ErrorKind,
    pub file: Option<PathBuf>,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    Io(IoError),
    InvalidModuleName,
    Parser(String),
    DuplicateImport(String),
    Subprocess(String, Option<String>),
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Error { kind, file: None }
    }

    pub(crate) fn with_file<P>(kind: ErrorKind, file: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Error {
            kind,
            file: Some(file.into()),
        }
    }

    pub(crate) fn set_file<P>(mut self, file: P) -> Self
    where
        P: Into<PathBuf>,
    {
        self.file = Some(file.into());
        self
    }

    pub(crate) fn io(e: IoError) -> Self {
        Self::new(ErrorKind::Io(e))
    }

    pub(crate) fn parser<R>(e: PestError<R>) -> Self
    where
        R: pest::RuleType,
    {
        Self::new(ErrorKind::Parser(e.to_string()))
    }

    pub(crate) fn duplicate_import<S>(module: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(ErrorKind::DuplicateImport(module.into()))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Io(e) => f.write_fmt(format_args!("io error ({})", e)),
            ErrorKind::InvalidModuleName => f.write_str("invalid module name"),
            ErrorKind::Parser(e) => e.fmt(f),
            ErrorKind::DuplicateImport(m) => {
                f.write_fmt(format_args!("duplicate import \"{}\"", m))
            }
            ErrorKind::Subprocess(n, Some(o)) => {
                f.write_fmt(format_args!("subprocess \"{}\" failed:\n{}", n, o))
            }
            ErrorKind::Subprocess(n, None) => {
                f.write_fmt(format_args!("subprocess \"{}\" failed", n))
            }
        }
    }
}

impl StdError for Error {}
