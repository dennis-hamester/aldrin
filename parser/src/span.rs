use crate::grammar::Rule;
use pest::iterators::Pair;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineCol {
    pub line: usize,
    pub column: usize,
}

impl LineCol {
    fn from_pest(pos: &pest::Position) -> Self {
        let (line, column) = pos.line_col();
        Self { line, column }
    }

    fn dummy() -> Self {
        Self { line: 0, column: 0 }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub index: usize,
    pub line_col: LineCol,
}

impl Position {
    pub(crate) fn from_pest(pos: &pest::Position) -> Self {
        Self {
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

        Self { index, line_col }
    }

    fn dummy() -> Self {
        Self {
            index: 0,
            line_col: LineCol::dummy(),
        }
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
        Self {
            from: Position::from_pest(&span.start_pos()),
            to: Position::from_pest(&span.end_pos()),
        }
    }

    pub(crate) fn dummy() -> Self {
        Self {
            from: Position::dummy(),
            to: Position::dummy(),
        }
    }
}
