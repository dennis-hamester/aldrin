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

mod error;
mod schema;

use schema::{ModuleName, Schema};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use error::{Error, ErrorKind};

#[derive(Debug)]
pub struct Generator {
    schema: Schema,
    options: Options,
    imported: HashMap<ModuleName, Schema>,
}

impl Generator {
    pub fn from_path<P>(path: P, options: Options) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Generator {
            schema: Schema::parse_file(path)?,
            options,
            imported: HashMap::new(),
        })
    }
}

#[derive(Debug, Default)]
pub struct Options {
    include_dirs: Vec<PathBuf>,
}

impl Options {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_include_dirs<I>(&mut self, dirs: I)
    where
        I: IntoIterator,
        I::Item: Into<PathBuf>,
    {
        self.include_dirs = dirs.into_iter().map(Into::into).collect();
    }

    pub fn add_include_dir<P>(&mut self, path: P)
    where
        P: Into<PathBuf>,
    {
        self.include_dirs.push(path.into());
    }
}
