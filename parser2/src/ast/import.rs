use super::{Comment, Ident, KwImport, PunctSemicolon};
use crate::Span;
use crate::error::ParseError;
use crate::lexer::Token;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::group;
use chumsky::{IterParser, Parser};

#[derive(Debug, Clone)]
pub struct Import {
    pub comments: Vec<Comment>,
    pub schema: Ident,
}

impl Import {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Comment::parser().repeated().collect(),
            KwImport::parser(),
            Ident::parser(),
            PunctSemicolon::parser(),
        ))
        .map(|(comments, _, schema, _)| Self { comments, schema })
    }
}
