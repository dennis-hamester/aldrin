// Copyright (c) 2019 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use pest::error::Error as PestError;
use semver::SemVerError;
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
    DuplicateDefinition(String),
    InvalidVersion(String),
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

    pub(crate) fn duplicate_definition<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(ErrorKind::DuplicateDefinition(name.into()))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

impl From<SemVerError> for Error {
    fn from(e: SemVerError) -> Self {
        Self::new(ErrorKind::InvalidVersion(e.to_string()))
    }
}

impl StdError for Error {}
