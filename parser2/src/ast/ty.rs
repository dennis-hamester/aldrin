use super::{
    KwBox, KwMap, KwOption, KwReceiver, KwResult, KwSender, KwSet, KwVec, LitInt, NamedRef,
    PunctAngClose, PunctAngOpen, PunctArrow, PunctComma, PunctSemicolon, PunctSquClose,
    PunctSquOpen,
};
use crate::error::{Expected, ParseError};
use crate::lexer::Token;
use crate::{Span, Spanned};
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::{choice, group};
use chumsky::recursive::recursive;

#[derive(Debug, Clone)]
pub enum Type {
    Option(Box<Self>),
    Box(Box<Self>),
    Vec(Box<Self>),
    Map(Box<Self>, Box<Self>),
    Set(Box<Self>),
    Sender(Box<Self>),
    Receiver(Box<Self>),
    Result(Box<Self>, Box<Self>),
    Array(Box<Self>, ArrayLen),
    Named(NamedRef),
}

impl Type {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        recursive(|ty| {
            choice((
                Self::option(ty.clone()),
                Self::box_ty(ty.clone()),
                Self::vec(ty.clone()),
                Self::map(ty.clone()),
                Self::set(ty.clone()),
                Self::sender(ty.clone()),
                Self::receiver(ty.clone()),
                Self::result(ty.clone()),
                Self::array(ty),
                NamedRef::parser().map(Self::Named),
            ))
            .labelled(Expected::TYPE)
        })
    }

    fn option<'a, I>(
        ty: impl Parser<'a, I, Self, Err<ParseError>> + Clone,
    ) -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            KwOption::parser(),
            PunctAngOpen::parser(),
            ty,
            PunctAngClose::parser(),
        ))
        .map(|(_, _, ty, _)| Self::Option(Box::new(ty)))
    }

    fn box_ty<'a, I>(
        ty: impl Parser<'a, I, Self, Err<ParseError>> + Clone,
    ) -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            KwBox::parser(),
            PunctAngOpen::parser(),
            ty,
            PunctAngClose::parser(),
        ))
        .map(|(_, _, ty, _)| Self::Box(Box::new(ty)))
    }

    fn vec<'a, I>(
        ty: impl Parser<'a, I, Self, Err<ParseError>> + Clone,
    ) -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            KwVec::parser(),
            PunctAngOpen::parser(),
            ty,
            PunctAngClose::parser(),
        ))
        .map(|(_, _, ty, _)| Self::Vec(Box::new(ty)))
    }

    fn map<'a, I>(
        ty: impl Parser<'a, I, Self, Err<ParseError>> + Clone,
    ) -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            KwMap::parser(),
            PunctAngOpen::parser(),
            ty.clone(),
            PunctArrow::parser(),
            ty,
            PunctAngClose::parser(),
        ))
        .map(|(_, _, key, _, val, _)| Self::Map(Box::new(key), Box::new(val)))
    }

    fn set<'a, I>(
        ty: impl Parser<'a, I, Self, Err<ParseError>> + Clone,
    ) -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            KwSet::parser(),
            PunctAngOpen::parser(),
            ty,
            PunctAngClose::parser(),
        ))
        .map(|(_, _, ty, _)| Self::Set(Box::new(ty)))
    }

    fn sender<'a, I>(
        ty: impl Parser<'a, I, Self, Err<ParseError>> + Clone,
    ) -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            KwSender::parser(),
            PunctAngOpen::parser(),
            ty,
            PunctAngClose::parser(),
        ))
        .map(|(_, _, ty, _)| Self::Sender(Box::new(ty)))
    }

    fn receiver<'a, I>(
        ty: impl Parser<'a, I, Self, Err<ParseError>> + Clone,
    ) -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            KwReceiver::parser(),
            PunctAngOpen::parser(),
            ty,
            PunctAngClose::parser(),
        ))
        .map(|(_, _, ty, _)| Self::Receiver(Box::new(ty)))
    }

    fn result<'a, I>(
        ty: impl Parser<'a, I, Self, Err<ParseError>> + Clone,
    ) -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            KwResult::parser(),
            PunctAngOpen::parser(),
            ty.clone(),
            PunctComma::parser(),
            ty,
            PunctAngClose::parser(),
        ))
        .map(|(_, _, ok, _, err, _)| Self::Result(Box::new(ok), Box::new(err)))
    }

    fn array<'a, I>(
        ty: impl Parser<'a, I, Self, Err<ParseError>> + Clone,
    ) -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            PunctSquOpen::parser(),
            ty,
            PunctSemicolon::parser(),
            ArrayLen::parser(),
            PunctSquClose::parser(),
        ))
        .map(|(_, ty, _, len, _)| Self::Array(Box::new(ty), len))
    }

    pub fn is_option(&self) -> bool {
        matches!(self, Self::Option(_))
    }

    pub fn as_option(&self) -> Option<&Self> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Option(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn is_box(&self) -> bool {
        matches!(self, Self::Box(_))
    }

    pub fn as_box(&self) -> Option<&Self> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Box(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn is_vec(&self) -> bool {
        matches!(self, Self::Vec(_))
    }

    pub fn as_vec(&self) -> Option<&Self> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Vec(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(..))
    }

    pub fn as_map(&self) -> Option<(&Self, &Self)> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Map(ty, val) => Some((ty, val)),
            _ => None,
        }
    }

    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set(_))
    }

    pub fn as_set(&self) -> Option<&Self> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Set(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn is_sender(&self) -> bool {
        matches!(self, Self::Sender(_))
    }

    pub fn as_sender(&self) -> Option<&Self> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Sender(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn is_receiver(&self) -> bool {
        matches!(self, Self::Receiver(_))
    }

    pub fn as_receiver(&self) -> Option<&Self> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Receiver(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn is_result(&self) -> bool {
        matches!(self, Self::Result(..))
    }

    pub fn as_result(&self) -> Option<(&Self, &Self)> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Result(ok, err) => Some((ok, err)),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(..))
    }

    pub fn as_array(&self) -> Option<(&Self, ArrayLen)> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Array(ty, len) => Some((ty, *len)),
            _ => None,
        }
    }

    pub fn is_named(&self) -> bool {
        matches!(self, Self::Named(_))
    }

    pub fn as_named(&self) -> Option<&NamedRef> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Named(ty) => Some(ty),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ArrayLen {
    Literal(LitInt),
    Named(NamedRef),
}

impl ArrayLen {
    fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        choice((
            LitInt::parser().map(Self::Literal),
            NamedRef::parser().map(Self::Named),
        ))
    }

    pub fn is_literal(self) -> bool {
        matches!(self, Self::Literal(_))
    }

    pub fn as_literal(self) -> Option<LitInt> {
        match self {
            Self::Literal(len) => Some(len),
            _ => None,
        }
    }

    pub fn is_named(self) -> bool {
        matches!(self, Self::Named(_))
    }

    pub fn as_named(self) -> Option<NamedRef> {
        match self {
            Self::Named(len) => Some(len),
            _ => None,
        }
    }
}

impl Spanned for ArrayLen {
    fn span(&self) -> Span {
        match self {
            Self::Literal(len) => len.span(),
            Self::Named(len) => len.span(),
        }
    }
}
