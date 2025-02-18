use super::kw;
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token, Type};

pub(super) struct EvFallbackItem {
    ident: Ident,
    variant: Ident,
    ty: Type,
}

impl EvFallbackItem {
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn gen_next_event_match_arm(&self, event: &Ident) -> TokenStream {
        let variant = &self.variant;

        quote! {
            _ => {
                break ::std::task::Poll::Ready(::std::option::Option::Some(
                    ::std::result::Result::Ok(#event::#variant(ev.into_unknown_event())),
                ));
            }
        }
    }

    pub fn gen_variant(&self) -> TokenStream {
        let variant = &self.variant;
        let ty = &self.ty;
        quote! { #variant(#ty), }
    }

    pub fn layout(&self) -> TokenStream {
        let name = self.ident.unraw().to_string();
        quote! { .event_fallback(#name) }
    }
}

impl Parse for EvFallbackItem {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::event>()?;
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let ty = input.parse()?;
        input.parse::<Token![;]>()?;

        let variant = Ident::new_raw(
            &ident.unraw().to_string().to_upper_camel_case(),
            ident.span(),
        );

        Ok(Self { ident, variant, ty })
    }
}
