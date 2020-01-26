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

mod enum_def;
mod grammar;
mod ident;
mod service;
mod struct_def;
mod types;

use crate::error::{Error, ErrorKind};
use enum_def::{Enum, InlineEnum};
use grammar::{Grammar, Rule};
use ident::{Ident, ModuleName};
use pest::iterators::Pair;
use pest::Parser;
use service::Service;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use struct_def::{InlineStruct, Struct};
use types::{Type, TypeOrInline};

#[derive(Debug)]
pub struct Schema {
    path: Option<PathBuf>,
    module: ModuleName,
    imported_modules: HashSet<ModuleName>,
    definitions: Vec<Definition>,
}

impl Schema {
    pub fn parse_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        let module = path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| Error::with_file(ErrorKind::InvalidModuleName, path))?;

        let mut schema_string = String::new();
        File::open(path)
            .and_then(|mut f| f.read_to_string(&mut schema_string))
            .map_err(|e| Error::with_file(ErrorKind::Io(e), path))?;

        let mut this = Self::parse_string(schema_string, module).map_err(|e| e.set_file(path))?;
        this.path = Some(path.to_path_buf());
        Ok(this)
    }

    fn parse_string<S, M>(s: S, module: M) -> Result<Self, Error>
    where
        S: AsRef<str>,
        M: Into<String>,
    {
        let s = s.as_ref();
        let parsed = Grammar::parse(Rule::file, s).map_err(Error::parser)?;

        let mut schema = Schema {
            path: None,
            module: ModuleName::from_string(module)?,
            imported_modules: HashSet::new(),
            definitions: Vec::new(),
        };

        for pair in parsed {
            match pair.as_rule() {
                Rule::EOI => break,
                Rule::import_stmt => schema.import_stmt(pair)?,
                Rule::struct_def => schema.struct_def(pair)?,
                Rule::enum_def => schema.enum_def(pair)?,
                Rule::service_def => schema.service_def(pair)?,
                _ => unreachable!(),
            }
        }

        Ok(schema)
    }

    fn import_stmt(&mut self, pair: Pair<Rule>) -> Result<(), Error> {
        assert_eq!(pair.as_rule(), Rule::import_stmt);
        let mut pairs = pair.into_inner();
        let imported_module = pairs.next().unwrap().as_str();
        let unique = self
            .imported_modules
            .insert(ModuleName::from_string(imported_module)?);
        if unique {
            Ok(())
        } else {
            Err(Error::duplicate_import(imported_module))
        }
    }

    fn struct_def(&mut self, pair: Pair<Rule>) -> Result<(), Error> {
        assert_eq!(pair.as_rule(), Rule::struct_def);
        self.definitions
            .push(Definition::Struct(Struct::from_struct_def(pair)?));
        Ok(())
    }

    fn enum_def(&mut self, pair: Pair<Rule>) -> Result<(), Error> {
        assert_eq!(pair.as_rule(), Rule::enum_def);
        self.definitions
            .push(Definition::Enum(Enum::from_enum_def(pair)?));
        Ok(())
    }

    fn service_def(&mut self, pair: Pair<Rule>) -> Result<(), Error> {
        assert_eq!(pair.as_rule(), Rule::service_def);
        self.definitions
            .push(Definition::Service(Service::from_service_def(pair)?));
        Ok(())
    }
}

#[derive(Debug)]
pub enum Definition {
    Struct(Struct),
    Enum(Enum),
    Service(Service),
}

impl Definition {
    pub fn name(&self) -> &Ident {
        match self {
            Definition::Struct(s) => &s.name,
            Definition::Enum(e) => &e.name,
            Definition::Service(s) => &s.name,
        }
    }
}
