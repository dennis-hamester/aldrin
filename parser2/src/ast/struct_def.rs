use super::{
    Attribute, Comment, DocComment, Ident, KwFallback, KwRequired, KwStruct, LitInt, Prelude,
    PunctAt, PunctCurClose, PunctCurOpen, PunctEq, PunctSemicolon, Type,
};
use crate::Span;
use crate::error::ParseError;
use crate::lexer::Token;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::{choice, group};
use chumsky::{IterParser, Parser};

#[derive(Debug, Clone)]
pub struct Struct {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub attributes: Vec<Attribute>,
    pub name: Ident,
    pub fields: Vec<Field>,
    pub fallback: Option<FallbackField>,
}

impl Struct {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Prelude::parser(),
            KwStruct::parser(),
            Ident::parser(),
            PunctCurOpen::parser(),
            Field::parser().repeated().collect(),
            FallbackField::parser().or_not(),
            PunctCurClose::parser(),
        ))
        .map(|(prelude, _, name, _, fields, fallback, _)| Self {
            comments: prelude.comments,
            doc_comments: prelude.doc_comments,
            attributes: prelude.attributes,
            name,
            fields,
            fallback,
        })
        .boxed()
    }
}

#[derive(Debug, Clone)]
pub struct InlineStruct {
    pub doc_comments: Vec<DocComment>,
    pub attributes: Vec<Attribute>,
    pub fields: Vec<Field>,
    pub fallback: Option<FallbackField>,
}

impl InlineStruct {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            KwStruct::parser(),
            PunctCurOpen::parser(),
            Prelude::inline_parser(),
            Field::parser().repeated().collect(),
            FallbackField::parser().or_not(),
            PunctCurClose::parser(),
        ))
        .map(|(_, _, prelude, fields, fallback, _)| Self {
            doc_comments: prelude.doc_comments,
            attributes: prelude.attributes,
            fields,
            fallback,
        })
        .boxed()
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub required: bool,
    pub name: Ident,
    pub id: LitInt,
    pub ty: Type,
}

impl Field {
    fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        let kw_required_name = choice((
            KwRequired::parser()
                .then(Ident::parser())
                .map(|(_, name)| (true, name)),
            Ident::parser().map(|name| (false, name)),
        ));

        group((
            Prelude::comment_parser(),
            kw_required_name,
            PunctAt::parser(),
            LitInt::parser(),
            PunctEq::parser(),
            Type::parser(),
            PunctSemicolon::parser(),
        ))
        .map(|(prelude, (required, name), _, id, _, ty, _)| Self {
            comments: prelude.comments,
            doc_comments: prelude.doc_comments,
            required,
            name,
            id,
            ty,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FallbackField {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub name: Ident,
}

impl FallbackField {
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
