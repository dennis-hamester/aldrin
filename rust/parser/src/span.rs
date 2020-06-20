use crate::grammar::Rule;
use pest::iterators::Pair;
use std::iter::Skip;
use std::str::Lines;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineCol {
    pub line: usize,
    pub column: usize,
}

impl LineCol {
    fn from_pest(pos: &pest::Position) -> Self {
        let (line, column) = pos.line_col();
        LineCol { line, column }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub index: usize,
    pub line_col: LineCol,
}

impl Position {
    pub(crate) fn from_pest(pos: &pest::Position) -> Self {
        Position {
            index: pos.pos(),
            line_col: LineCol::from_pest(pos),
        }
    }

    pub(crate) fn from_pest_error<R>(err: &pest::error::Error<R>) -> Self {
        use pest::error::{InputLocation, LineColLocation};

        let index = match err.location {
            InputLocation::Pos(index) => index,
            InputLocation::Span((from, _)) => from,
        };

        let line_col = match err.line_col {
            LineColLocation::Pos((line, column)) => LineCol { line, column },
            LineColLocation::Span((line, column), _) => LineCol { line, column },
        };

        Position { index, line_col }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub from: Position,
    pub to: Position,
}

impl Span {
    pub(crate) fn from_pair(pair: &Pair<Rule>) -> Self {
        let span = pair.as_span();
        Span {
            from: Position::from_pest(&span.start_pos()),
            to: Position::from_pest(&span.end_pos()),
        }
    }

    pub fn lines(self, text: &str) -> SpanLines {
        SpanLines {
            span: self,
            lines: text.lines().skip(self.from.line_col.line - 1),
        }
    }
}

#[derive(Debug)]
pub struct SpanLines<'a> {
    span: Span,
    lines: Skip<Lines<'a>>,
}

impl<'a> Iterator for SpanLines<'a> {
    type Item = (&'a str, Span);

    fn next(&mut self) -> Option<Self::Item> {
        if self.span.from == self.span.to {
            None
        } else if self.span.from.line_col.line < self.span.to.line_col.line {
            let line = self.lines.next()?;

            let span = Span {
                from: self.span.from,
                to: Position {
                    index: self.span.from.index + line.len(),
                    line_col: LineCol {
                        line: self.span.from.line_col.line,
                        column: line.len() + 1,
                    },
                },
            };

            self.span.from.index = 0;
            self.span.from.line_col.line += 1;
            self.span.from.line_col.column = 1;

            Some((line, span))
        } else {
            let line = self.lines.next()?;
            let span = self.span;
            self.span.from = self.span.to;
            Some((line, span))
        }
    }
}
