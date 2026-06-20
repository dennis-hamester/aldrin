use super::{
    Comment, DocComment, Event, FallbackEvent, FallbackFunction, Function, Ident, KwService,
    KwUuid, KwVersion, LitInt, LitUuid, Prelude, PunctCurClose, PunctCurOpen, PunctEq,
    PunctSemicolon,
};
use crate::Span;
use crate::error::ParseError;
use crate::lexer::Token;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::{choice, group};
use chumsky::{IterParser, Parser};

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
}
