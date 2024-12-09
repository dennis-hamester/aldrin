use crate::error::ExpectedIdentFoundReserved;
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct Ident {
    span: Span,
    value: String,
}

impl Ident {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::ident);

        Self {
            span: Span::from_pair(&pair),
            value: pair.as_str().to_owned(),
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        ExpectedIdentFoundReserved::validate(self, validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
