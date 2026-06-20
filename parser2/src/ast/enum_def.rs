use super::{
    Attribute, Comment, DocComment, Ident, KwEnum, KwFallback, LitInt, Prelude, PunctAt,
    PunctCurClose, PunctCurOpen, PunctEq, PunctSemicolon, Type,
};
use crate::Span;
use crate::error::ParseError;
use crate::lexer::Token;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::group;
use chumsky::{IterParser, Parser};

#[derive(Debug, Clone)]
pub struct Enum {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub attributes: Vec<Attribute>,
    pub name: Ident,
    pub variants: Vec<Variant>,
    pub fallback: Option<FallbackVariant>,
}

impl Enum {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Prelude::parser(),
            KwEnum::parser(),
            Ident::parser(),
            PunctCurOpen::parser(),
            Variant::parser().repeated().collect(),
            FallbackVariant::parser().or_not(),
            PunctCurClose::parser(),
        ))
        .map(|(prelude, _, name, _, variants, fallback, _)| Self {
            comments: prelude.comments,
            doc_comments: prelude.doc_comments,
            attributes: prelude.attributes,
            name,
            variants,
            fallback,
        })
        .boxed()
    }
}

#[derive(Debug, Clone)]
pub struct InlineEnum {
    pub doc_comments: Vec<DocComment>,
    pub attributes: Vec<Attribute>,
    pub variants: Vec<Variant>,
    pub fallback: Option<FallbackVariant>,
}

impl InlineEnum {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            KwEnum::parser(),
            PunctCurOpen::parser(),
            Prelude::inline_parser(),
            Variant::parser().repeated().collect(),
            FallbackVariant::parser().or_not(),
            PunctCurClose::parser(),
        ))
        .map(|(_, _, prelude, variants, fallback, _)| Self {
            doc_comments: prelude.doc_comments,
            attributes: prelude.attributes,
            variants,
            fallback,
        })
        .boxed()
    }
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub name: Ident,
    pub id: LitInt,
    pub ty: Option<Type>,
}

impl Variant {
    fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Prelude::comment_parser(),
            Ident::parser(),
            PunctAt::parser(),
            LitInt::parser(),
            PunctEq::parser()
                .then(Type::parser())
                .map(|(_, ty)| ty)
                .or_not(),
            PunctSemicolon::parser(),
        ))
        .map(|(prelude, name, _, id, ty, _)| Self {
            comments: prelude.comments,
            doc_comments: prelude.doc_comments,
            name,
            id,
            ty,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FallbackVariant {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub name: Ident,
}

impl FallbackVariant {
    fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Prelude::comment_parser(),
            Ident::parser(),
            PunctEq::parser(),
            KwFallback::parser(),
            PunctSemicolon::parser(),
        ))
        .map(|(prelude, name, _, _, _)| Self {
            comments: prelude.comments,
            doc_comments: prelude.doc_comments,
            name,
        })
    }
}
