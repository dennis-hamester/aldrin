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
use super::{Ident, TypeOrInline};
use crate::error::Error;
use pest::iterators::Pair;

#[derive(Debug)]
pub struct Enum {
    pub name: Ident,
    pub variants: Vec<Variant>,
}

impl Enum {
    pub fn from_enum_def(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::enum_def);
        let mut pairs = pair.into_inner();
        let name = Ident::from_string(pairs.next().unwrap().as_str())?;
        let variants = enum_body(pairs.next().unwrap())?;
        Ok(Enum { name, variants })
    }
}

#[derive(Debug)]
pub struct InlineEnum {
    pub variants: Vec<Variant>,
}

impl InlineEnum {
    pub fn from_enum_inline(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::enum_inline);
        let mut pairs = pair.into_inner();
        let variants = enum_body(pairs.next().unwrap())?;
        Ok(InlineEnum { variants })
    }
}

#[derive(Debug)]
pub struct Variant {
    pub name: Ident,
    pub id: u32,
    pub variant_type: Option<TypeOrInline>,
}

fn enum_body(pair: Pair<Rule>) -> Result<Vec<Variant>, Error> {
    assert_eq!(pair.as_rule(), Rule::enum_body);
    let pairs = pair.into_inner();
    let mut res = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::enum_variant);
        let mut pairs = pair.into_inner();
        let name = Ident::from_string(pairs.next().unwrap().as_str())?;
        let id = pairs.next().unwrap().as_str().parse().unwrap();
        let variant_type = pairs
            .next()
            .map(TypeOrInline::from_type_name_or_inline)
            .transpose()?;
        res.push(Variant {
            name,
            id,
            variant_type,
        });
    }
    Ok(res)
}
