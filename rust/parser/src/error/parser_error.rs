use super::Error;
use crate::grammar::Rule;
use crate::Position;
use std::collections::HashSet;

#[derive(Debug)]
pub struct ParserError {
    schema_name: String,
    pos: Position,
    expected: HashSet<Expected>,
}

impl ParserError {
    pub(crate) fn new<S>(schema_name: S, err: pest::error::Error<Rule>) -> Self
    where
        S: Into<String>,
    {
        use pest::error::ErrorVariant;

        let pos = Position::from_pest_error(&err);

        let positives = match err.variant {
            ErrorVariant::ParsingError { positives, .. } => positives,
            ErrorVariant::CustomError { .. } => unreachable!(),
        };
        let expected = positives.into_iter().map(Expected::from_pest).collect();

        ParserError {
            schema_name: schema_name.into(),
            pos,
            expected,
        }
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    pub fn expected(&self) -> &HashSet<Expected> {
        &self.expected
    }
}

impl From<ParserError> for Error {
    fn from(e: ParserError) -> Self {
        Error::Parser(e)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expected {
    Eof,
    Ident,
    ImportStmt,
    LitInt,
    LitString,
    LitUuid,
    SchemaName,
    Terminator,
}

impl Expected {
    pub(crate) fn from_pest(rule: Rule) -> Self {
        match rule {
            Rule::EOI => Expected::Eof,
            Rule::ident => Expected::Ident,
            Rule::kw_import => Expected::ImportStmt,
            Rule::lit_int => Expected::LitInt,
            Rule::lit_string => Expected::LitString,
            Rule::lit_uuid => Expected::LitUuid,
            Rule::schema_name => Expected::SchemaName,
            Rule::term => Expected::Terminator,

            Rule::COMMENT
            | Rule::WHITESPACE
            | Rule::file
            | Rule::import_stmt
            | Rule::lit_pos_nonzero_int
            | Rule::lit_string_char
            | Rule::ws => unreachable!(),
        }
    }
}
