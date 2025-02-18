use crate::grammar::Rule;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct SchemaName {
    span: Span,
    value: String,
}

impl SchemaName {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::schema_name);

        Self {
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
