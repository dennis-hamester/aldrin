use super::{
    Comment, DocComment, Ident, KwArgs, KwErr, KwFallback, KwFn, KwOk, LitInt, Prelude, PunctAt,
    PunctCurClose, PunctCurOpen, PunctEq, PunctSemicolon, TypeOrInline,
};
use crate::Span;
use crate::error::ParseError;
use crate::lexer::Token;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::{choice, group};
use chumsky::{IterParser, Parser};

#[derive(Debug, Clone)]
pub struct Function {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub name: Ident,
    pub id: LitInt,
    pub args: Option<FunctionPart>,
    pub ok: Option<FunctionPart>,
    pub err: Option<FunctionPart>,
}

impl Function {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        let body_full = group((
            PunctCurOpen::parser(),
            FunctionPart::args_parser().or_not(),
            FunctionPart::ok_parser().or_not(),
            FunctionPart::err_parser().or_not(),
            PunctCurClose::parser(),
        ))
        .map(|(_, args, ok, err, _)| (args, ok, err));

        let body_ok = PunctEq::parser()
            .ignore_then(TypeOrInline::parser())
            .map(|ty| (None, Some(FunctionPart::from_ty(ty)), None));

        let body_empty = PunctSemicolon::parser().to((None, None, None));
        let body = choice((body_full, body_ok, body_empty));

        group((
            Prelude::comment_parser(),
            KwFn::parser(),
            Ident::parser(),
            PunctAt::parser(),
            LitInt::parser(),
            body,
        ))
        .map(|(prelude, _, name, _, id, (args, ok, err))| Self {
            comments: prelude.comments,
            doc_comments: prelude.doc_comments,
            name,
            id,
            args,
            ok,
            err,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FunctionPart {
    pub comments: Vec<Comment>,
    pub ty: TypeOrInline,
}

impl FunctionPart {
    fn args_parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Comment::parser().repeated().collect(),
            KwArgs::parser(),
            PunctEq::parser(),
            TypeOrInline::parser(),
        ))
        .map(|(comments, _, _, ty)| Self { comments, ty })
    }

    fn ok_parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Comment::parser().repeated().collect(),
            KwOk::parser(),
            PunctEq::parser(),
            TypeOrInline::parser(),
        ))
        .map(|(comments, _, _, ty)| Self { comments, ty })
    }

    fn err_parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Comment::parser().repeated().collect(),
            KwErr::parser(),
            PunctEq::parser(),
            TypeOrInline::parser(),
        ))
        .map(|(comments, _, _, ty)| Self { comments, ty })
    }

    fn from_ty(ty: TypeOrInline) -> Self {
        Self {
            comments: Vec::new(),
            ty,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FallbackFunction {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub name: Ident,
}

impl FallbackFunction {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Prelude::comment_parser(),
            KwFn::parser(),
            Ident::parser(),
            PunctEq::parser(),
            KwFallback::parser(),
            PunctSemicolon::parser(),
        ))
        .map(|(prelude, _, name, _, _, _)| Self {
            comments: prelude.comments,
            doc_comments: prelude.doc_comments,
            name,
        })
    }
}
