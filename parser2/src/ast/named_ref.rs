use super::{Ident, PunctScope};
use crate::error::ParseError;
use crate::lexer::Token;
use crate::{Span, Spanned};
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::{choice, group};

#[derive(Debug, Copy, Clone)]
pub enum NamedRef {
    Internal(Ident),
    External(Ident, Ident),
}

impl NamedRef {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        choice((
            group((Ident::parser(), PunctScope::parser(), Ident::parser()))
                .map(|(schema, _, ty)| Self::External(schema, ty)),
            Ident::parser().map(Self::Internal),
        ))
    }
}

impl Spanned for NamedRef {
    fn span(&self) -> Span {
        match self {
            Self::Internal(ty) => ty.span(),
            Self::External(schema, ty) => schema.span().extend(ty.span()),
        }
    }
}
