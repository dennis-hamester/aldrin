use chumsky::span::Span as ChumskySpan;
use std::ops::Range;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub fn extend(self, other: Self) -> Self {
        debug_assert!(self.end <= other.start);

        Self {
            start: self.start,
            end: other.end,
        }
    }
}

impl Spanned for Span {
    fn span(&self) -> Self {
        *self
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl ChumskySpan for Span {
    type Context = ();
    type Offset = usize;

    fn new(_context: Self::Context, range: Range<Self::Offset>) -> Self {
        range.into()
    }

    fn context(&self) -> Self::Context {}

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}

pub trait Spanned {
    fn span(&self) -> Span;

    fn source<'a>(&self, source: &'a str) -> &'a str {
        let span = self.span();
        &source[span.start..span.end]
    }
}
