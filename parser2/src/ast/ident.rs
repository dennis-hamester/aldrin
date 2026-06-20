use crate::error::{Expected, ParseError};
use crate::lexer::Token;
use crate::{Span, Spanned};
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::any;

#[derive(Debug, Copy, Clone)]
pub struct Ident {
    pub span: Span,
}

impl Ident {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        any()
            .filter(|tok| matches!(tok, Token::Ident(_)))
            .map_with(|_, extra| Self { span: extra.span() })
            .labelled(Expected::IDENT)
    }
}

impl Spanned for Ident {
    fn span(&self) -> Span {
        self.span
    }
}
