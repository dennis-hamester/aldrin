use super::{kw, EvItem, FnItem, Options};
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Result, Token, Type};

pub(super) enum ServiceItem {
    Event(EvItem),
    Function(FnItem),
}

impl ServiceItem {
    pub fn as_event(&self) -> Option<&EvItem> {
        match self {
            Self::Event(ev) => Some(ev),
            Self::Function(_) => None,
        }
    }

    pub fn as_function(&self) -> Option<&FnItem> {
        match self {
            Self::Event(_) => None,
            Self::Function(func) => Some(func),
        }
    }

    pub fn layout(&self, options: &Options) -> TokenStream {
        match self {
            Self::Event(ev) => ev.layout(options),
            Self::Function(func) => func.layout(options),
        }
    }

    pub fn add_references<'a>(&'a self, references: &mut HashSet<&'a Type>) {
        match self {
            Self::Event(ev) => ev.add_references(references),
            Self::Function(func) => func.add_references(references),
        }
    }
}

impl Parse for ServiceItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let begin = input.fork();
        begin.call(Attribute::parse_outer)?;

        let lookahead = begin.lookahead1();
        if lookahead.peek(kw::event) {
            input.parse().map(Self::Event)
        } else if lookahead.peek(Token![fn]) {
            input.parse().map(Self::Function)
        } else {
            Err(lookahead.error())
        }
    }
}
