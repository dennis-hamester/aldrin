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

    pub fn is_service(&self) -> bool {
        matches!(self, Self::Service(_))
    }

    pub fn as_service(&self) -> Option<&Service> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Service(def) => Some(def),
            _ => None,
        }
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
    }

    pub fn as_struct(&self) -> Option<&Struct> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Struct(def) => Some(def),
            _ => None,
        }
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum(_))
    }

    pub fn as_enum(&self) -> Option<&Enum> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Enum(def) => Some(def),
            _ => None,
        }
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const(_))
    }

    pub fn as_const(&self) -> Option<&Const> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Const(def) => Some(def),
            _ => None,
        }
    }

    pub fn is_newtype(&self) -> bool {
        matches!(self, Self::Newtype(_))
    }

    pub fn as_newtype(&self) -> Option<&Newtype> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Newtype(def) => Some(def),
            _ => None,
        }
    }
}
