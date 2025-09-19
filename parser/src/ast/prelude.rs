use super::{Attribute, Comment, DocString};
use crate::grammar::Rule;
use pest::iterators::Pairs;
use std::mem;

pub(crate) struct Prelude {
    comment: Comment,
    doc: DocString,
    doc_inline: DocString,
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
        let mut comment = Comment::new();
        let mut doc = DocString::new();
        let mut doc_inline = DocString::new();
        let mut attrs = Vec::new();
        let mut attrs_inline = Vec::new();

        while let Some(pair) = pairs.peek() {
            match pair.as_rule() {
                Rule::comment if allow_comments => comment.push(pair),
                Rule::doc_string if !inline => doc.push(pair),
                Rule::doc_string_inline if inline => doc_inline.push_inline(pair),
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

    pub(crate) fn take_comment(&mut self) -> Comment {
        mem::replace(&mut self.comment, Comment::new())
    }

    pub(crate) fn take_doc(&mut self) -> DocString {
        mem::replace(&mut self.doc, DocString::new())
    }

    pub(crate) fn take_inline_doc(&mut self) -> DocString {
        mem::replace(&mut self.doc_inline, DocString::new())
    }

    pub(crate) fn take_attrs(&mut self) -> Vec<Attribute> {
        mem::take(&mut self.attrs)
    }

    pub(crate) fn take_attrs_inline(&mut self) -> Vec<Attribute> {
        mem::take(&mut self.attrs_inline)
    }
}
