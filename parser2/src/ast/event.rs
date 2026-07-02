use super::{
    Comment, DocComment, Ident, KwEvent, KwFallback, LitInt, Prelude, PunctAt, PunctEq,
    PunctSemicolon, Schema, Service, TypeOrInline,
};
use crate::error::ParseError;
use crate::lexer::Token;
use crate::visitor::{FieldCtx, InlineCtx, VariantCtx};
use crate::{Span, Visitor};
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::{choice, group};
use std::ops::ControlFlow;

#[derive(Debug, Clone)]
pub struct Event {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub name: Ident,
    pub id: LitInt,
    pub ty: Option<TypeOrInline>,
}

impl Event {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        let ty = choice((
            PunctSemicolon::parser().to(None),
            PunctEq::parser()
                .then(TypeOrInline::parser())
                .map(|(_, ty)| Some(ty)),
        ));

        group((
            Prelude::comment_parser(),
            KwEvent::parser(),
            Ident::parser(),
            PunctAt::parser(),
            LitInt::parser(),
            ty,
        ))
        .map(|(prelude, _, name, _, id, ty)| Self {
            comments: prelude.comments,
            doc_comments: prelude.doc_comments,
            name,
            id,
            ty,
        })
    }

    pub(crate) fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
        schema: &'a Schema,
        service: &'a Service,
    ) -> ControlFlow<T::Output> {
        visitor.event(schema, service, self)?;

        match self.ty {
            Some(TypeOrInline::Struct(ref struct_def)) => struct_def.visit_impl(
                visitor,
                schema,
                service,
                InlineCtx::Event(self),
                FieldCtx::Event(service, self, struct_def),
            )?,

            Some(TypeOrInline::Enum(ref enum_def)) => enum_def.visit_impl(
                visitor,
                schema,
                service,
                InlineCtx::Event(self),
                VariantCtx::Event(service, self, enum_def),
            )?,

            Some(TypeOrInline::Type(_)) | None => {}
        }

        ControlFlow::Continue(())
    }
}

#[derive(Debug, Clone)]
pub struct FallbackEvent {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub name: Ident,
}

impl FallbackEvent {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Prelude::comment_parser(),
            KwEvent::parser(),
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
        visitor.fallback_event(schema, service, self)
    }
}
