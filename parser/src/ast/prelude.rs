use super::{Attribute, Comment, DocString};
use crate::grammar::Rule;
use pest::iterators::Pairs;
use std::mem;

pub(crate) struct Prelude {
    comment: Vec<Comment>,
    doc: Vec<DocString>,
    doc_inline: Vec<DocString>,
    attrs: Vec<Attribute>,
    attrs_inline: Vec<Attribute>,
}

impl Prelude {
    pub(crate) fn schema(pairs: &mut Pairs<Rule>) -> Self {
        Self::new_impl(pairs, true, true)
    }

    pub(crate) fn regular(pairs: &mut Pairs<Rule>) -> Self {
        Self::new_impl(pairs, true, false)
    }

    pub(crate) fn inline(pairs: &mut Pairs<Rule>) -> Self {
        Self::new_impl(pairs, false, true)
    }

    fn new_impl(pairs: &mut Pairs<Rule>, allow_comments: bool, inline: bool) -> Self {
        let mut comment = Vec::new();
        let mut doc = Vec::new();
        let mut doc_inline = Vec::new();
        let mut attrs = Vec::new();
        let mut attrs_inline = Vec::new();

        while let Some(pair) = pairs.peek() {
            match pair.as_rule() {
                Rule::comment if allow_comments => comment.push(Comment::parse(pair)),
                Rule::doc_string if !inline => doc.push(DocString::parse(pair)),
                Rule::doc_string_inline if inline => doc_inline.push(DocString::parse_inline(pair)),
                Rule::attribute if !inline => attrs.push(Attribute::parse(pair)),

                Rule::attribute_inline if inline => {
                    attrs_inline.push(Attribute::parse_inline(pair))
                }

                _ => break,
            }

            pairs.next();
        }

        Self {
            comment,
            doc,
            doc_inline,
            attrs,
            attrs_inline,
        }
    }

    pub(crate) fn take_comment(&mut self) -> Vec<Comment> {
        mem::take(&mut self.comment)
    }

    pub(crate) fn take_doc(&mut self) -> Vec<DocString> {
        mem::take(&mut self.doc)
    }

    pub(crate) fn take_inline_doc(&mut self) -> Vec<DocString> {
        mem::take(&mut self.doc_inline)
    }

    pub(crate) fn take_attrs(&mut self) -> Vec<Attribute> {
        mem::take(&mut self.attrs)
    }

    pub(crate) fn take_attrs_inline(&mut self) -> Vec<Attribute> {
        mem::take(&mut self.attrs_inline)
    }
}
