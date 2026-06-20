use super::{
    Ident, PunctComma, PunctExclamation, PunctHash, PunctParClose, PunctParOpen, PunctSquClose,
    PunctSquOpen,
};
use crate::Span;
use crate::error::{Expected, ParseError};
use crate::lexer::Token;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::group;
use chumsky::{IterParser, Parser};

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Ident,
    pub args: Vec<Ident>,
}

impl Attribute {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            PunctHash::parser(),
            PunctSquOpen::parser(),
            Ident::parser(),
            PunctParOpen::parser(),
            Self::args_parser(),
            PunctParClose::parser(),
            PunctSquClose::parser(),
        ))
        .map(|(_, _, name, _, args, _, _)| Self { name, args })
        .labelled(Expected::ATTRIBUTE)
    }

    pub(crate) fn inline_parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            PunctHash::parser(),
            PunctExclamation::parser(),
            PunctSquOpen::parser(),
            Ident::parser(),
            PunctParOpen::parser(),
            Self::args_parser(),
            PunctParClose::parser(),
            PunctSquClose::parser(),
        ))
        .map(|(_, _, _, name, _, args, _, _)| Self { name, args })
        .labelled(Expected::INLINE_ATTRIBUTE)
    }

    fn args_parser<'a, I>() -> impl Parser<'a, I, Vec<Ident>, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        Ident::parser()
            .separated_by(PunctComma::parser())
            .allow_trailing()
            .collect()
    }
}
