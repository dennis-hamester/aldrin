use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token, Type};

pub(super) struct FnFallbackItem {
    ident: Ident,
    variant: Ident,
    ty: Type,
}

impl FnFallbackItem {
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn variant(&self) -> &Ident {
        &self.variant
    }

    pub fn gen_variant(&self) -> TokenStream {
        let variant = &self.variant;
        let ty = &self.ty;

        quote! {
            #[allow(dead_code)]
            #variant(#ty),
        }
    }

    pub fn gen_next_call_match_arm(&self, function: &Ident) -> TokenStream {
        let variant = &self.variant;

        quote! {
            _ => ::std::task::Poll::Ready(
                ::std::option::Option::Some(
                    ::std::result::Result::Ok(#function::#variant(call.into_unknown_call()))
                ),
            ),
        }
    }

    pub fn layout(&self) -> TokenStream {
        let name = self.ident.unraw().to_string();

        quote! { .function_fallback(#name) }
    }
}

impl Parse for FnFallbackItem {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![fn]>()?;
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
