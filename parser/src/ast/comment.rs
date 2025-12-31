use crate::Span;
use crate::grammar::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct Comment {
    span: Span,
    value: String,
}

impl Comment {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::comment);

        Self {
            span: Span::from_pair(&pair),
            value: pair.as_str().to_owned(),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn span_inner(&self) -> Span {
        let value = &self.value[2..];

        Span {
            start: self.span.start + 2 + value.starts_with(' ') as usize,
            end: self.span.end - value.len() + value.trim_end().len(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn value_inner(&self) -> &str {
        let value = &self.value[2..];
        value.strip_prefix(' ').unwrap_or(value).trim_end()
    }
}
