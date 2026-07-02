use super::{
    Comment, DocComment, Event, FallbackEvent, FallbackFunction, Function, Ident, KwService,
    KwUuid, KwVersion, LitInt, LitUuid, Prelude, PunctCurClose, PunctCurOpen, PunctEq,
    PunctSemicolon, Schema,
};
use crate::error::ParseError;
use crate::lexer::Token;
use crate::{Span, Visitor};
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::{choice, group};
use chumsky::{IterParser, Parser};
use std::ops::ControlFlow;

#[derive(Debug, Clone)]
pub struct Service {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub name: Ident,
    pub uuid_comments: Vec<Comment>,
    pub uuid: LitUuid,
    pub version_comments: Vec<Comment>,
    pub version: LitInt,
    pub items: Vec<ServiceItem>,
    pub fallback_fn: Option<FallbackFunction>,
    pub fallback_event: Option<FallbackEvent>,
}

impl Service {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        let fallbacks = choice((
            FallbackFunction::parser()
                .then(FallbackEvent::parser().or_not())
                .map(|(fallback_fn, fallback_event)| (Some(fallback_fn), fallback_event)),
            FallbackEvent::parser()
                .then(FallbackFunction::parser().or_not())
                .map(|(fallback_event, fallback_fn)| (fallback_fn, Some(fallback_event))),
        ))
        .or_not()
        .map(|fallbacks| fallbacks.unwrap_or((None, None)));

        group((
            Prelude::comment_parser(),
            KwService::parser(),
            Ident::parser(),
            PunctCurOpen::parser(),
            Comment::parser().repeated().collect(),
            KwUuid::parser(),
            PunctEq::parser(),
            LitUuid::parser(),
            PunctSemicolon::parser(),
            Comment::parser().repeated().collect(),
            KwVersion::parser(),
            PunctEq::parser(),
            LitInt::parser(),
            PunctSemicolon::parser(),
            ServiceItem::parser().repeated().collect(),
            fallbacks,
            PunctCurClose::parser(),
        ))
        .map(
            |(
                prelude,
                _,
                name,
                _,
                uuid_comments,
                _,
                _,
                uuid,
                _,
                version_comments,
                _,
                _,
                version,
                _,
                items,
                (fallback_fn, fallback_event),
                _,
            )| Self {
                comments: prelude.comments,
                doc_comments: prelude.doc_comments,
                name,
                uuid_comments,
                uuid,
                version_comments,
                version,
                items,
                fallback_fn,
                fallback_event,
            },
        )
        .boxed()
    }

    pub(crate) fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
        schema: &'a Schema,
    ) -> ControlFlow<T::Output> {
        visitor.service(schema, self)?;

        for item in &self.items {
            item.visit_impl(visitor, schema, self)?;
        }

        if let Some(ref fallback) = self.fallback_fn {
            fallback.visit_impl(visitor, schema, self)?;
        }

        if let Some(ref fallback) = self.fallback_event {
            fallback.visit_impl(visitor, schema, self)?;
        }

        ControlFlow::Continue(())
    }
}

#[derive(Debug, Clone)]
#[expect(clippy::large_enum_variant)]
pub enum ServiceItem {
    Fn(Function),
    Event(Event),
}

impl ServiceItem {
    fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        choice((
            Function::parser().map(Self::Fn),
            Event::parser().map(Self::Event),
        ))
    }

    pub fn is_fn(&self) -> bool {
        matches!(self, Self::Fn(_))
    }

    pub fn as_fn(&self) -> Option<&Function> {
        match self {
            Self::Fn(item) => Some(item),
            _ => None,
        }
    }

    pub fn is_event(&self) -> bool {
        matches!(self, Self::Event(_))
    }

    pub fn as_event(&self) -> Option<&Event> {
        match self {
            Self::Event(item) => Some(item),
            _ => None,
        }
    }

    fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
        schema: &'a Schema,
        service: &'a Service,
    ) -> ControlFlow<T::Output> {
        match self {
            Self::Fn(item) => item.visit_impl(visitor, schema, service),
            Self::Event(item) => item.visit_impl(visitor, schema, service),
        }
    }
}
