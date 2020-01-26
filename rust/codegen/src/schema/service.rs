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
use semver::Version;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct Service {
    pub name: Ident,
    pub uuid: Uuid,
    pub ver: Version,
    pub elems: Vec<ServiceElement>,
}

impl Service {
    pub fn from_service_def(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::service_def);
        let mut pairs = pair.into_inner();
        let name = Ident::from_string(pairs.next().unwrap().as_str())?;
        let uuid = pairs.next().unwrap().as_str().parse().unwrap();
        let ver = pairs.next().unwrap().as_str().parse()?;

        let mut this = Service {
            name,
            uuid,
            ver,
            elems: Vec::new(),
        };

        for pair in pairs {
            match pair.as_rule() {
                Rule::fn_def => this.fn_def(pair)?,
                Rule::event_def => this.event_def(pair)?,
                _ => unreachable!(),
            }
        }

        Ok(this)
    }

    fn fn_def(&mut self, pair: Pair<Rule>) -> Result<(), Error> {
        assert_eq!(pair.as_rule(), Rule::fn_def);
        self.elems
            .push(ServiceElement::Function(Function::from_fn_def(pair)?));
        Ok(())
    }

    fn event_def(&mut self, pair: Pair<Rule>) -> Result<(), Error> {
        assert_eq!(pair.as_rule(), Rule::event_def);
        self.elems
            .push(ServiceElement::Event(Event::from_event_def(pair)?));
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum ServiceElement {
    Function(Function),
    Event(Event),
}

#[derive(Debug)]
pub(crate) struct FunctionDeprecation {
    pub since: Version,
    pub reason: String,
}

impl FunctionDeprecation {
    fn from_attr_depr(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::attr_depr);
        let mut pairs = pair.into_inner();
        let since = pairs.next().unwrap().as_str().parse()?;
        let reason = pairs.next().unwrap().as_str().to_owned();
        Ok(FunctionDeprecation { since, reason })
    }
}

#[derive(Debug)]
pub(crate) struct Function {
    pub name: Ident,
    pub id: u32,
    pub deprecation: Option<FunctionDeprecation>,
    pub args: Option<TypeOrInline>,
    pub ok: Option<TypeOrInline>,
    pub err: Option<TypeOrInline>,
}

impl Function {
    fn from_fn_def(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::fn_def);
        let mut pairs = pair.into_inner();

        let mut pair = pairs.next().unwrap();
        let deprecation = if pair.as_rule() == Rule::attr_depr {
            let depr = FunctionDeprecation::from_attr_depr(pair)?;
            pair = pairs.next().unwrap();
            Some(depr)
        } else {
            None
        };

        let name = Ident::from_string(pair.as_str())?;
        let id = pairs.next().unwrap().as_str().parse().unwrap();

        let mut args = None;
        let mut ok = None;
        let mut err = None;
        for _ in 0..3 {
            if let Some(pair) = pairs.next() {
                match pair.as_rule() {
                    Rule::fn_args => {
                        args = Some(TypeOrInline::from_type_name_or_inline(
                            pair.into_inner().next().unwrap(),
                        )?);
                    }
                    Rule::fn_ok => {
                        ok = Some(TypeOrInline::from_type_name_or_inline(
                            pair.into_inner().next().unwrap(),
                        )?);
                    }
                    Rule::fn_err => {
                        err = Some(TypeOrInline::from_type_name_or_inline(
                            pair.into_inner().next().unwrap(),
                        )?);
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(Function {
            name,
            id,
            deprecation,
            args,
            ok,
            err,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Event {
    pub name: Ident,
    pub id: u32,
    pub event_type: Option<TypeOrInline>,
}

impl Event {
    fn from_event_def(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::event_def);
        let mut pairs = pair.into_inner();
        let name = Ident::from_string(pairs.next().unwrap().as_str())?;
        let id = pairs.next().unwrap().as_str().parse().unwrap();
        let event_type = pairs
            .next()
            .map(TypeOrInline::from_type_name_or_inline)
            .transpose()?;
        Ok(Event {
            name,
            id,
            event_type,
        })
    }
}
