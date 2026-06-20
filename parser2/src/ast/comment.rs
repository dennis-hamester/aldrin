use crate::error::{Expected, ParseError};
use crate::lexer::Token;
use crate::{Span, Spanned};
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::just;

#[derive(Debug, Copy, Clone)]
pub struct Comment {
    pub span: Span,
}

impl Comment {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        just(Token::Comment)
            .map_with(|_, extra| Self { span: extra.span() })
            .labelled(Expected::COMMENT)
    }
}

impl Spanned for Comment {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DocComment {
    pub span: Span,
}

impl DocComment {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        just(Token::DocComment)
            .map_with(|_, extra| Self { span: extra.span() })
            .labelled(Expected::DOC_COMMENT)
    }

    pub(crate) fn inline_parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        just(Token::InlineDocComment)
            .map_with(|_, extra| Self { span: extra.span() })
            .labelled(Expected::INLINE_DOC_COMMENT)
    }
}

impl Spanned for DocComment {
    fn span(&self) -> Span {
        self.span
    }
}
