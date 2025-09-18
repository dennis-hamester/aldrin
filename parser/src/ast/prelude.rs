use super::{Attribute, DocString};
use crate::grammar::Rule;
use pest::iterators::Pairs;
use std::mem;

pub(crate) struct Prelude {
    doc: DocString,
    attrs: Vec<Attribute>,
}

impl Prelude {
    pub(crate) fn new(pairs: &mut Pairs<Rule>) -> Self {
        let mut doc = DocString::new();
        let mut attrs = Vec::new();

        while let Some(pair) = pairs.peek() {
            match pair.as_rule() {
                Rule::doc_string => doc.push(pair),
                Rule::attribute => attrs.push(Attribute::parse(pair)),
                _ => break,
            }

            pairs.next();
        }

        Self { doc, attrs }
    }

    pub(crate) fn take_doc(&mut self) -> DocString {
        mem::replace(&mut self.doc, DocString::new())
    }

    pub(crate) fn take_attrs(&mut self) -> Vec<Attribute> {
        mem::take(&mut self.attrs)
    }
}
