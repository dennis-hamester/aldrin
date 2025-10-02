use super::{Attribute, DocString, Ident, Prelude, TypeName};
use crate::error::RecursiveNewtype;
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::{BrokenDocLink, NonCamelCaseNewtype};
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct NewtypeDef {
    span: Span,
    comment: Option<String>,
    doc: Vec<DocString>,
    attrs: Vec<Attribute>,
    name: Ident,
    target_type: TypeName,
}

impl NewtypeDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::newtype_def);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip =.

        let target_type_pair = pairs.next().unwrap();
        let target_type = TypeName::parse(target_type_pair);

        Self {
            span,
            comment: prelude.take_comment().into(),
            doc: prelude.take_doc(),
            attrs: prelude.take_attrs(),
            name,
            target_type,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        BrokenDocLink::validate(&self.doc, validate);
        RecursiveNewtype::validate(self, validate);
        NonCamelCaseNewtype::validate(self, validate);

        self.name.validate(true, validate);
        self.target_type.validate(false, validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    pub fn doc(&self) -> &[DocString] {
        &self.doc
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attrs
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn target_type(&self) -> &TypeName {
        &self.target_type
    }
}
