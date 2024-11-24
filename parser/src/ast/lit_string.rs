use crate::grammar::Rule;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct LitString {
    span: Span,
    value: String,
}

impl LitString {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::lit_string);

        let value = pair.as_str();
        let value = value[1..value.len() - 1].to_owned();

        Self {
            span: Span::from_pair(&pair),
            value,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
