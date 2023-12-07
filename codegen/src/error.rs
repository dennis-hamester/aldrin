use diffy::{ApplyError, ParsePatchError};
use std::fmt;
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Subprocess(#[from] SubprocessError),

    #[error(transparent)]
    ParsePatch(#[from] ParsePatchError),

    #[error(transparent)]
    ApplyPatch(#[from] ApplyError),
}

#[derive(Error, Debug)]
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
