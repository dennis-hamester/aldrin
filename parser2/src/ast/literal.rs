use crate::error::{Expected, ParseError};
use crate::lexer::Token;
use crate::{Span, Spanned};
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::just;

#[derive(Debug, Copy, Clone)]
pub struct LitString {
    pub span: Span,
}

impl LitString {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        just(Token::LitString)
            .map_with(|_, extra| Self { span: extra.span() })
            .labelled(Expected::LIT_STRING)
    }
}

impl Spanned for LitString {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LitInt {
    pub span: Span,
}

impl LitInt {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        just(Token::LitInt)
            .map_with(|_, extra| Self { span: extra.span() })
            .labelled(Expected::LIT_INT)
    }
}

impl Spanned for LitInt {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LitUuid {
    pub span: Span,
}

impl LitUuid {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        just(Token::LitUuid)
            .map_with(|_, extra| Self { span: extra.span() })
            .labelled(Expected::LIT_UUID)
    }
}

impl Spanned for LitUuid {
    fn span(&self) -> Span {
        self.span
    }
}
