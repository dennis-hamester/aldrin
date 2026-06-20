use super::{InlineEnum, InlineStruct, PunctSemicolon, Type};
use crate::Span;
use crate::error::ParseError;
use crate::lexer::Token;
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::choice;

#[derive(Debug, Clone)]
pub enum TypeOrInline {
    Type(Type),
    Struct(InlineStruct),
    Enum(InlineEnum),
}

impl TypeOrInline {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        let ty = Type::parser()
            .then(PunctSemicolon::parser())
            .map(|(ty, _)| Self::Type(ty));

        choice((
            ty,
            InlineStruct::parser().map(Self::Struct),
            InlineEnum::parser().map(Self::Enum),
        ))
    }
}
