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
}

impl Spanned for ArrayLen {
    fn span(&self) -> Span {
        match self {
            Self::Literal(len) => len.span(),
            Self::Named(len) => len.span(),
        }
    }
}
