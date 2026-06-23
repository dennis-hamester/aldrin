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

    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type(_))
    }

    pub fn as_type(&self) -> Option<&Type> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Type(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
    }

    pub fn as_struct(&self) -> Option<&InlineStruct> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Struct(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum(_))
    }

    pub fn as_enum(&self) -> Option<&InlineEnum> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Enum(ty) => Some(ty),
            _ => None,
        }
    }
}
