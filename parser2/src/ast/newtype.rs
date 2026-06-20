use super::{
    Attribute, Comment, DocComment, Ident, KwNewtype, Prelude, PunctEq, PunctSemicolon, Type,
};
use crate::Span;
use crate::error::ParseError;
use crate::lexer::Token;
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::group;

#[derive(Debug, Clone)]
pub struct Newtype {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub attributes: Vec<Attribute>,
    pub name: Ident,
    pub ty: Type,
}

impl Newtype {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Prelude::parser(),
            KwNewtype::parser(),
            Ident::parser(),
            PunctEq::parser(),
            Type::parser(),
            PunctSemicolon::parser(),
        ))
        .map(|(prelude, _, name, _, ty, _)| Self {
            comments: prelude.comments,
            doc_comments: prelude.doc_comments,
            attributes: prelude.attributes,
            name,
            ty,
        })
        .boxed()
    }
}
