// Copyright (c) 2020 Dennis Hamester <dennis.hamester@gmail.com>
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

use super::grammar::Rule;
use super::{Ident, Type};
use crate::error::Error;
use pest::iterators::Pair;

#[derive(Debug)]
pub(crate) struct Struct {
    pub name: Ident,
    pub fields: Vec<Field>,
}

impl Struct {
    pub fn from_struct_def(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::struct_def);
        let mut pairs = pair.into_inner();
        let name = Ident::from_string(pairs.next().unwrap().as_str())?;
        let fields = struct_body(pairs.next().unwrap())?;
        Ok(Struct { name, fields })
    }
}

#[derive(Debug)]
pub(crate) struct InlineStruct {
    pub fields: Vec<Field>,
}

impl InlineStruct {
    pub fn from_struct_inline(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::struct_inline);
        let mut pairs = pair.into_inner();
        let fields = struct_body(pairs.next().unwrap())?;
        Ok(InlineStruct { fields })
    }
}

#[derive(Debug)]
pub(crate) struct Field {
    pub name: Ident,
    pub id: u32,
    pub field_type: Type,
    pub required: bool,
}

fn struct_body(pair: Pair<Rule>) -> Result<Vec<Field>, Error> {
    assert_eq!(pair.as_rule(), Rule::struct_body);
    let pairs = pair.into_inner();
    let mut res = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::struct_field);
        let mut pairs = pair.into_inner();

        let mut pair = pairs.next().unwrap();
        let required = if pair.as_rule() == Rule::struct_field_req {
            pair = pairs.next().unwrap();
            true
        } else {
            false
        };
        let name = Ident::from_string(pair.as_str())?;

        let id = pairs.next().unwrap().as_str().parse().unwrap();
        let field_type = Type::from_type_name(pairs.next().unwrap())?;
        res.push(Field {
            name,
            id,
            field_type,
            required,
        });
    }
    Ok(res)
}
