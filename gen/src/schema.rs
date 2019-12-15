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

mod grammar;

use crate::Error;
use grammar::Rule;
use pest::iterators::Pairs;
use pest::Parser;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Schema {
    path: PathBuf,
    imported_modules: HashSet<String>,
}

impl Schema {
    pub fn parse_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        let mut schema_file = String::new();
        File::open(path)
            .map_err(|e| format!("failed to open \"{}\" ({})", path.display(), e))?
            .read_to_string(&mut schema_file)
            .map_err(|e| format!("failed to read \"{}\" ({})", path.display(), e))?;

        let parsed = grammar::Grammar::parse(grammar::Rule::file, &schema_file)
            .map_err(|e| format!("failed to parse \"{}\" ({})", path.display(), e))?;
        println!("{:#?}", parsed);

        let mut schema = Schema {
            path: path.to_path_buf(),
            imported_modules: HashSet::new(),
        };

        for pair in parsed {
            match pair.as_rule() {
                Rule::EOI => break,

                Rule::import_stmt => schema.import_stmt(pair.into_inner())?,

                _ => {}
            }
        }

        Ok(schema)
    }

    fn import_stmt(&mut self, mut pairs: Pairs<Rule>) -> Result<(), Error> {
        let imported_module = pairs.next().unwrap().as_str();
        let unique = self.imported_modules.insert(imported_module.to_owned());
        if unique {
            Ok(())
        } else {
            Err(format!("duplicate imported module \"{}\"", imported_module).into())
        }
    }
}
