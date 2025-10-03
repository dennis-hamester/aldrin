use crate::grammar::Rule;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct DocString {
    span: Span,
    value: String,
}

impl DocString {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::doc_string);
        Self::parse_impl(pair)
    }

    pub(crate) fn parse_inline(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::doc_string_inline);
        Self::parse_impl(pair)
    }

    fn parse_impl(pair: Pair<Rule>) -> Self {
        Self {
            span: Span::from_pair(&pair),
            value: pair.as_str().to_owned(),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn span_inner(&self) -> Span {
        let value = &self.value[3..];

        Span {
            start: self.span.start + 3 + value.starts_with(' ') as usize,
            end: self.span.end - value.len() + value.trim_end().len(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn value_inner(&self) -> &str {
        let value = &self.value[3..];
        value.strip_prefix(' ').unwrap_or(value).trim_end()
    }
}
