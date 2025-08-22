use super::kw;
use crate::doc_string::DocString;
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token, Type};

pub(super) struct EvFallbackItem {
    doc: DocString,
    ident: Ident,
    variant: Ident,
    ty: Type,
}

impl EvFallbackItem {
    pub(crate) fn ident(&self) -> &Ident {
        &self.ident
    }

    pub(crate) fn variant(&self) -> &Ident {
        &self.variant
    }

    pub(crate) fn gen_next_event_match_arm(&self, event: &Ident) -> TokenStream {
        let variant = &self.variant;

        quote! {
            _ => {
                break ::std::task::Poll::Ready(::std::option::Option::Some(
                    ::std::result::Result::Ok(#event::#variant(ev.into_unknown_event())),
                ));
            }
        }
    }

    pub(crate) fn gen_variant(&self) -> TokenStream {
        let variant = &self.variant;
        let ty = &self.ty;
        let doc = &self.doc;

        quote! {
            #doc
            #[allow(dead_code)]
            #variant(#ty),
        }
    }

    pub(crate) fn layout(&self) -> TokenStream {
        let name = self.ident.unraw().to_string();

        quote! { .event_fallback(#name) }
    }
}

impl Parse for EvFallbackItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let doc = input.parse()?;
        input.parse::<kw::event>()?;
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let ty = input.parse()?;
        input.parse::<Token![;]>()?;

        let variant = Ident::new_raw(
            &ident.unraw().to_string().to_upper_camel_case(),
            ident.span(),
        );

        Ok(Self {
            doc,
            ident,
            variant,
            ty,
        })
    }
}
