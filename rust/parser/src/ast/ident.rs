use crate::grammar::Rule;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug)]
pub struct Ident {
    span: Span,
    value: String,
}

impl Ident {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::ident);
        Ident {
            span: Span::from_pair(&pair),
            value: pair.as_str().to_owned(),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
