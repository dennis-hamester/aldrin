use crate::Span;
use crate::grammar::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct LitString {
    span: Span,
    value: String,
}

impl LitString {
    pub(crate) fn parse(pair: &Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::lit_string);

        Self {
            span: Span::from_pair(pair),
            value: pair.as_str().to_owned(),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn span_inner(&self) -> Span {
        Span {
            start: self.span.start + 1,
            end: self.span.end - 1,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn value_inner(&self) -> &str {
        &self.value[1..self.value.len() - 1]
    }
}
