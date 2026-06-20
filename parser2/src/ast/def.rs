use super::{Const, Enum, Newtype, Service, Struct};
use crate::Span;
use crate::error::ParseError;
use crate::lexer::Token;
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::choice;

#[derive(Debug, Clone)]
pub enum Definition {
    Service(Service),
    Struct(Struct),
    Enum(Enum),
    Const(Const),
    Newtype(Newtype),
}

impl Definition {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        choice((
            Service::parser().map(Self::Service),
            Struct::parser().map(Self::Struct),
            Enum::parser().map(Self::Enum),
            Const::parser().map(Self::Const),
            Newtype::parser().map(Self::Newtype),
        ))
    }
}
