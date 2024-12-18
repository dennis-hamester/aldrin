use crate::grammar::Rule;
use crate::Span;
use pest::iterators::Pair;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LitUuid {
    span: Span,
    value: Uuid,
}

impl LitUuid {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::lit_uuid);

        Self {
            span: Span::from_pair(&pair),
            value: pair.as_str().parse().unwrap(),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn value(&self) -> Uuid {
        self.value
    }
}
