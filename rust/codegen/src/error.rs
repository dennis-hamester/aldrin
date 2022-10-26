use diffy::{ApplyError, ParsePatchError};
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Subprocess(SubprocessError),
    ParsePatch(ParsePatchError),
    ApplyPatch(ApplyError),
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}

impl From<ParsePatchError> for Error {
    fn from(e: ParsePatchError) -> Self {
        Error::ParsePatch(e)
    }
}

impl From<ApplyError> for Error {
    fn from(e: ApplyError) -> Self {
        Error::ApplyPatch(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => e.fmt(f),
            Error::Subprocess(e) => e.fmt(f),
            Error::ParsePatch(e) => e.fmt(f),
            Error::ApplyPatch(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {}

#[derive(Debug)]
pub struct SubprocessError {
    pub(crate) command: String,
    pub(crate) code: Option<i32>,
    pub(crate) stderr: Option<String>,
}

impl SubprocessError {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn code(&self) -> Option<i32> {
        self.code
    }

    pub fn stderr(&self) -> Option<&str> {
        self.stderr.as_deref()
    }
}

impl From<SubprocessError> for Error {
    fn from(e: SubprocessError) -> Self {
        Error::Subprocess(e)
    }
}

impl fmt::Display for SubprocessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "subprocess `{}` failed", self.command)?;

        if let Some(code) = self.code {
            write!(f, " with code {code}")?;
        }

        if f.alternate() {
            if let Some(ref stderr) = self.stderr {
                writeln!(f, ":\n{stderr}")?;
            }
        }

        Ok(())
    }
}

impl StdError for SubprocessError {}
