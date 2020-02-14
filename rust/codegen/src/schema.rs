mod consts;
mod enum_def;
mod grammar;
mod ident;
mod service;
mod struct_def;
mod types;

use crate::error::{Error, ErrorKind};
use pest::iterators::Pair;
use pest::Parser;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub(crate) use consts::*;
pub(crate) use enum_def::*;
pub(crate) use grammar::*;
pub(crate) use ident::*;
pub(crate) use service::*;
pub(crate) use struct_def::*;
pub(crate) use types::*;

#[derive(Debug)]
pub(crate) struct Schema {
    pub path: Option<PathBuf>,
    pub module: ModuleName,
    pub imported_modules: HashSet<ModuleName>,
    pub definitions: Vec<Definition>,
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
                Rule::const_def => schema.const_def(pair)?,
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

    fn const_def(&mut self, pair: Pair<Rule>) -> Result<(), Error> {
        assert_eq!(pair.as_rule(), Rule::const_def);
        self.definitions
            .push(Definition::Const(Const::from_const_def(pair)?));
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum Definition {
    Struct(Struct),
    Enum(Enum),
    Service(Service),
    Const(Const),
}
