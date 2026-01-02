use super::{EvFallbackItem, EvItem, FnFallbackItem, FnItem, Options, kw};
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Ident, Result, Token, Type};

#[expect(clippy::large_enum_variant)]
pub(super) enum ServiceItem {
    Event(EvItem),
    EventFallback(EvFallbackItem),
    Function(FnItem),
    FunctionFallback(FnFallbackItem),
}

impl ServiceItem {
    pub(crate) fn as_event(&self) -> Option<&EvItem> {
        match self {
            Self::Event(ev) => Some(ev),
            Self::EventFallback(_) | Self::Function(_) | Self::FunctionFallback(_) => None,
        }
    }

    pub(crate) fn as_fallback_event(&self) -> Option<&EvFallbackItem> {
        match self {
            Self::EventFallback(ev) => Some(ev),
            Self::Event(_) | Self::Function(_) | Self::FunctionFallback(_) => None,
        }
    }

    pub(crate) fn as_function(&self) -> Option<&FnItem> {
        match self {
            Self::Function(func) => Some(func),
            Self::Event(_) | Self::EventFallback(_) | Self::FunctionFallback(_) => None,
        }
    }

    pub(crate) fn as_fallback_function(&self) -> Option<&FnFallbackItem> {
        match self {
            Self::FunctionFallback(func) => Some(func),
            Self::Event(_) | Self::EventFallback(_) | Self::Function(_) => None,
        }
    }

    pub(crate) fn layout(&self, options: &Options) -> TokenStream {
        match self {
            Self::Event(ev) => ev.layout(options),
            Self::EventFallback(ev) => ev.layout(options),
            Self::Function(func) => func.layout(options),
            Self::FunctionFallback(func) => func.layout(options),
        }
    }

    pub(crate) fn add_references<'a>(&'a self, references: &mut HashSet<&'a Type>) {
        match self {
            Self::Event(ev) => ev.add_references(references),
            Self::Function(func) => func.add_references(references),
            Self::EventFallback(_) | Self::FunctionFallback(_) => {}
        }
    }

    fn parse_ev(input: ParseStream) -> Result<Self> {
        let begin = input.fork();
        begin.call(Attribute::parse_outer)?;

        begin.parse::<kw::event>()?;
        begin.parse::<Ident>()?;

        let lookahead = begin.lookahead1();
        if lookahead.peek(Token![@]) {
            input.parse().map(Self::Event)
        } else if lookahead.peek(Token![=]) {
            input.parse().map(Self::EventFallback)
        } else {
            Err(lookahead.error())
        }
    }

    fn parse_fn(input: ParseStream) -> Result<Self> {
        let begin = input.fork();
        begin.call(Attribute::parse_outer)?;

        begin.parse::<Token![fn]>()?;
        begin.parse::<Ident>()?;

        let lookahead = begin.lookahead1();
        if lookahead.peek(Token![@]) {
            input.parse().map(Self::Function)
        } else if lookahead.peek(Token![=]) {
            input.parse().map(Self::FunctionFallback)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for ServiceItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let begin = input.fork();
        begin.call(Attribute::parse_outer)?;

        let lookahead = begin.lookahead1();
        if lookahead.peek(kw::event) {
            input.call(Self::parse_ev)
        } else if lookahead.peek(Token![fn]) {
            input.call(Self::parse_fn)
        } else {
            Err(lookahead.error())
        }
    }
}
