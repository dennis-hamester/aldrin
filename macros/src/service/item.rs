use super::{kw, EvItem, FnFallbackItem, FnItem, Options};
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Ident, Result, Token, Type};

pub(super) enum ServiceItem {
    Event(EvItem),
    Function(FnItem),
    FunctionFallback(FnFallbackItem),
}

impl ServiceItem {
    pub fn as_event(&self) -> Option<&EvItem> {
        match self {
            Self::Event(ev) => Some(ev),
            Self::Function(_) => None,
            Self::FunctionFallback(_) => None,
        }
    }

    pub fn as_function(&self) -> Option<&FnItem> {
        match self {
            Self::Event(_) => None,
            Self::Function(func) => Some(func),
            Self::FunctionFallback(_) => None,
        }
    }

    pub fn as_fallback_function(&self) -> Option<&FnFallbackItem> {
        match self {
            Self::Event(_) => None,
            Self::Function(_) => None,
            Self::FunctionFallback(func) => Some(func),
        }
    }

    pub fn layout(&self, options: &Options) -> TokenStream {
        match self {
            Self::Event(ev) => ev.layout(options),
            Self::Function(func) => func.layout(options),
            Self::FunctionFallback(func) => func.layout(),
        }
    }

    pub fn add_references<'a>(&'a self, references: &mut HashSet<&'a Type>) {
        match self {
            Self::Event(ev) => ev.add_references(references),
            Self::Function(func) => func.add_references(references),
            Self::FunctionFallback(_) => {}
        }
    }

    fn parse_ev(input: ParseStream) -> Result<Self> {
        input.parse().map(Self::Event)
    }

    fn parse_fn(input: ParseStream) -> Result<Self> {
        let begin = input.fork();

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
