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
    pub(crate) fn from_pest(span: &pest::Span) -> Self {
        Span {
            from: Position::from_pest(&span.start_pos()),
            to: Position::from_pest(&span.end_pos()),
        }
    }
}
