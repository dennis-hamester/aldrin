use crate::grammar::Rule;
use pest::iterators::Pair;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub(crate) fn from_pair(pair: &Pair<Rule>) -> Self {
        let span = pair.as_span();

        Self {
            start: span.start(),
            end: span.end(),
        }
    }

    pub(crate) fn dummy() -> Self {
        Self { start: 0, end: 0 }
    }
}
