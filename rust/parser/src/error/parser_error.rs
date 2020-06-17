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
    ConstDef,
    ConstValue,
    Eof,
    Ident,
    ImportStmt,
    LitInt,
    LitString,
    LitUuid,
    SchemaName,
    TokenEquals,
    TokenParClose,
    TokenParOpen,
    TokenTerm,
}

impl Expected {
    pub(crate) fn from_pest(rule: Rule) -> Self {
        match rule {
            Rule::EOI => Expected::Eof,
            Rule::const_value => Expected::ConstValue,
            Rule::ident => Expected::Ident,
            Rule::kw_const => Expected::ConstDef,
            Rule::kw_import => Expected::ImportStmt,
            Rule::lit_int => Expected::LitInt,
            Rule::lit_string => Expected::LitString,
            Rule::lit_uuid => Expected::LitUuid,
            Rule::schema_name => Expected::SchemaName,
            Rule::tok_eq => Expected::TokenEquals,
            Rule::tok_par_close => Expected::TokenParClose,
            Rule::tok_par_open => Expected::TokenParOpen,
            Rule::tok_term => Expected::TokenTerm,

            Rule::COMMENT
            | Rule::WHITESPACE
            | Rule::const_def
            | Rule::const_i16
            | Rule::const_i32
            | Rule::const_i64
            | Rule::const_i8
            | Rule::const_string
            | Rule::const_u16
            | Rule::const_u32
            | Rule::const_u64
            | Rule::const_u8
            | Rule::const_uuid
            | Rule::file
            | Rule::import_stmt
            | Rule::kw_i16
            | Rule::kw_i32
            | Rule::kw_i64
            | Rule::kw_i8
            | Rule::kw_string
            | Rule::kw_u16
            | Rule::kw_u32
            | Rule::kw_u64
            | Rule::kw_u8
            | Rule::kw_uuid
            | Rule::lit_pos_nonzero_int
            | Rule::lit_string_char
            | Rule::ws => unreachable!(),
        }
    }
}
