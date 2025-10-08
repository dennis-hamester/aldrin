use super::{ItemOptions, Options};
use aldrin_codegen::rust::names;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token, Type};

pub(super) struct FnFallbackItem {
    options: ItemOptions,
    ident: Ident,
    variant: Ident,
    ty: Type,
}

impl FnFallbackItem {
    pub(crate) fn ident(&self) -> &Ident {
        &self.ident
    }

    pub(crate) fn variant(&self) -> &Ident {
        &self.variant
    }

    pub(crate) fn gen_variant(&self) -> TokenStream {
        let variant = &self.variant;
        let ty = &self.ty;
        let doc = &self.options.doc();

        quote! {
            #doc
            #[allow(dead_code)]
            #variant(#ty),
        }
    }

    pub(crate) fn gen_next_call_match_arm(&self, call: &Ident) -> TokenStream {
        let variant = &self.variant;

        quote! {
            _ => ::std::task::Poll::Ready(
                ::std::option::Option::Some(
                    ::std::result::Result::Ok(#call::#variant(call.into_unknown_call()))
                ),
            ),
        }
    }

    pub(crate) fn layout(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let name = self.ident.unraw().to_string();
        let doc = self.options.doc_alt().to_introspection();

        quote! {
            .function_fallback(
                #krate::core::introspection::ir::FunctionFallbackIr::builder(#name)
                    #doc
                    .finish(),
            )
        }
    }
}

impl Parse for FnFallbackItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let options = input.parse()?;
        input.parse::<Token![fn]>()?;
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let ty = input.parse()?;
        input.parse::<Token![;]>()?;

        let variant = Ident::new_raw(
            &names::function_variant(&ident.unraw().to_string()),
            ident.span(),
        );

        Ok(Self {
            options,
            ident,
            variant,
            ty,
        })
    }
}
