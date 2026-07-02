use super::{Comment, Ident, KwImport, PunctSemicolon, Schema};
use crate::error::ParseError;
use crate::lexer::Token;
use crate::{Span, Visitor};
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::group;
use chumsky::{IterParser, Parser};
use std::ops::ControlFlow;

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

    pub(crate) fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
        schema: &'a Schema,
    ) -> ControlFlow<T::Output> {
        visitor.import(schema, self)
    }
}
