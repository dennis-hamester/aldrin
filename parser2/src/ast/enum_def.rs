use super::{
    Attribute, Comment, DocComment, Ident, KwEnum, KwFallback, LitInt, Prelude, PunctAt,
    PunctCurClose, PunctCurOpen, PunctEq, PunctSemicolon, Schema, Service, Type,
};
use crate::error::ParseError;
use crate::lexer::Token;
use crate::visitor::{InlineCtx, VariantCtx};
use crate::{Span, Visitor};
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::group;
use chumsky::{IterParser, Parser};
use std::ops::ControlFlow;

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

    pub(crate) fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
        schema: &'a Schema,
    ) -> ControlFlow<T::Output> {
        visitor.enum_def(schema, self)?;

        for var in &self.variants {
            var.visit_impl(visitor, schema, VariantCtx::Enum(self))?;
        }

        if let Some(ref fallback) = self.fallback {
            fallback.visit_impl(visitor, schema, VariantCtx::Enum(self))?;
        }

        ControlFlow::Continue(())
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

    pub(crate) fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
        schema: &'a Schema,
        service: &'a Service,
        inline_ctx: InlineCtx<'a>,
        variant_ctx: VariantCtx<'a>,
    ) -> ControlFlow<T::Output> {
        visitor.inline_enum(schema, service, inline_ctx, self)?;

        for var in &self.variants {
            var.visit_impl(visitor, schema, variant_ctx)?;
        }

        if let Some(ref fallback) = self.fallback {
            fallback.visit_impl(visitor, schema, variant_ctx)?;
        }

        ControlFlow::Continue(())
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

    fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
        schema: &'a Schema,
        ctx: VariantCtx<'a>,
    ) -> ControlFlow<T::Output> {
        visitor.variant(schema, ctx, self)
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

    fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
        schema: &'a Schema,
        ctx: VariantCtx<'a>,
    ) -> ControlFlow<T::Output> {
        visitor.fallback_variant(schema, ctx, self)
    }
}
