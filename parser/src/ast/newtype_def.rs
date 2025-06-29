use super::{Attribute, Ident, TypeName};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct NewtypeDef {
    span: Span,
    attrs: Vec<Attribute>,
    name: Ident,
    target_type: TypeName,
}

impl NewtypeDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::newtype_def);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        let mut attrs = Vec::new();
        for pair in &mut pairs {
            match pair.as_rule() {
                Rule::attribute => attrs.push(Attribute::parse(pair)),
                Rule::kw_newtype => break,
                _ => unreachable!(),
            }
        }

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip =.

        let target_type_pair = pairs.next().unwrap();
        let target_type = TypeName::parse(target_type_pair);

        Self {
            span,
            attrs,
            name,
            target_type,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        self.name.validate(validate);
        self.target_type.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
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
