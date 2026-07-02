use super::{
    Comment, DocComment, Ident, InlineEnum, InlineStruct, KwArgs, KwErr, KwFallback, KwFn, KwOk,
    LitInt, Prelude, PunctAt, PunctCurClose, PunctCurOpen, PunctEq, PunctSemicolon, Schema,
    Service, TypeOrInline,
};
use crate::error::ParseError;
use crate::lexer::Token;
use crate::visitor::{FieldCtx, InlineCtx, VariantCtx};
use crate::{Span, Visitor};
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::{choice, group};
use chumsky::{IterParser, Parser};
use std::ops::ControlFlow;

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

    pub(crate) fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
        schema: &'a Schema,
        service: &'a Service,
    ) -> ControlFlow<T::Output> {
        visitor.function(schema, service, self)?;

        if let Some(ref args) = self.args {
            args.visit_impl(
                visitor,
                schema,
                service,
                InlineCtx::FunctionArgs(self, args),
                |struct_def| FieldCtx::FunctionArgs(service, self, args, struct_def),
                |enum_def| VariantCtx::FunctionArgs(service, self, args, enum_def),
            )?;
        }

        if let Some(ref ok) = self.ok {
            ok.visit_impl(
                visitor,
                schema,
                service,
                InlineCtx::FunctionOk(self, ok),
                |struct_def| FieldCtx::FunctionOk(service, self, ok, struct_def),
                |enum_def| VariantCtx::FunctionOk(service, self, ok, enum_def),
            )?;
        }

        if let Some(ref err) = self.err {
            err.visit_impl(
                visitor,
                schema,
                service,
                InlineCtx::FunctionErr(self, err),
                |struct_def| FieldCtx::FunctionErr(service, self, err, struct_def),
                |enum_def| VariantCtx::FunctionErr(service, self, err, enum_def),
            )?;
        }

        ControlFlow::Continue(())
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

    fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
        schema: &'a Schema,
        service: &'a Service,
        inline_ctx: InlineCtx<'a>,
        field_ctx: impl FnOnce(&'a InlineStruct) -> FieldCtx<'a>,
        variant_ctx: impl FnOnce(&'a InlineEnum) -> VariantCtx<'a>,
    ) -> ControlFlow<T::Output> {
        match self.ty {
            TypeOrInline::Type(_) => ControlFlow::Continue(()),
            TypeOrInline::Struct(ref struct_def) => {
                struct_def.visit_impl(visitor, schema, service, inline_ctx, field_ctx(struct_def))
            }
            TypeOrInline::Enum(ref enum_def) => {
                enum_def.visit_impl(visitor, schema, service, inline_ctx, variant_ctx(enum_def))
            }
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

    pub(crate) fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
        schema: &'a Schema,
        service: &'a Service,
    ) -> ControlFlow<T::Output> {
        visitor.fallback_function(schema, service, self)
    }
}
