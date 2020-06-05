use super::Error;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct IoError {
    path: PathBuf,
    err: std::io::Error,
}

impl IoError {
    pub(crate) fn new<P>(path: P, err: std::io::Error) -> Self
    where
        P: Into<PathBuf>,
    {
        IoError {
            path: path.into(),
            err,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn io_error(&self) -> &std::io::Error {
        &self.err
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}
