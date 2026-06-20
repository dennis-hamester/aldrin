use super::{Attribute, Comment, DocComment};
use crate::Span;
use crate::error::ParseError;
use crate::lexer::Token;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::choice;
use chumsky::{IterParser, Parser};

pub(crate) struct Prelude {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub attributes: Vec<Attribute>,
}

impl Prelude {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        choice((
            Comment::parser().map(PreludeElement::Comment),
            DocComment::parser().map(PreludeElement::DocComment),
            Attribute::parser().map(PreludeElement::Attribute),
        ))
        .repeated()
        .collect()
        .map(|elems| Self::collect(elems, true, true))
    }

    pub(crate) fn inline_parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        choice((
            DocComment::inline_parser().map(PreludeElement::DocComment),
            Attribute::inline_parser().map(PreludeElement::Attribute),
        ))
        .repeated()
        .collect()
        .map(|elems| Self::collect(elems, false, true))
    }

    pub(crate) fn comment_parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        choice((
            Comment::parser().map(PreludeElement::Comment),
            DocComment::parser().map(PreludeElement::DocComment),
        ))
        .repeated()
        .collect()
        .map(|elems| Self::collect(elems, true, false))
    }

    pub(crate) fn schema_parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        Comment::parser()
            .repeated()
            .collect()
            .then(DocComment::inline_parser())
            .repeated()
            .collect()
            .map(Self::collect_schema)
    }

    fn collect(elems: Vec<PreludeElement>, with_comments: bool, with_attributes: bool) -> Self {
        let mut this = Self {
            comments: if with_comments {
                Vec::with_capacity(elems.len())
            } else {
                Vec::new()
            },

            doc_comments: Vec::with_capacity(elems.len()),

            attributes: if with_attributes {
                Vec::with_capacity(elems.len())
            } else {
                Vec::new()
            },
        };

        for elem in elems {
            match elem {
                PreludeElement::Comment(elem) => this.comments.push(elem),
                PreludeElement::DocComment(elem) => this.doc_comments.push(elem),
                PreludeElement::Attribute(elem) => this.attributes.push(elem),
            }
        }

        this
    }

    fn collect_schema(elems: Vec<(Vec<Comment>, DocComment)>) -> Self {
        let mut this = Self {
            comments: Vec::with_capacity(elems.len()),
            doc_comments: Vec::with_capacity(elems.len()),
            attributes: Vec::new(),
        };

        for (comments, doc_comment) in elems {
            this.comments.extend(comments);
            this.doc_comments.push(doc_comment);
        }

        this
    }
}

enum PreludeElement {
    Comment(Comment),
    DocComment(DocComment),
    Attribute(Attribute),
}
