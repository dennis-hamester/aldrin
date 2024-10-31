use super::{FnBody, Options};
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, Ident, LitInt, Result, Token, Type};

pub(super) struct FnItem {
    ident: Ident,
    variant: Ident,
    id: LitInt,
    body: FnBody,
}

impl FnItem {
    pub fn gen_call(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let ident = &self.ident;
        let id = &self.id;

        let (args, val) = match self.body.args() {
            Some(args) => (
                Some(quote! { , args: #krate::core::SerializeArg<#args> }),
                quote! { &args },
            ),

            None => (None, quote! { &() }),
        };

        let ok = match self.body.ok() {
            Some(ok) => quote! { #ok },
            None => quote! { () },
        };

        let err = match self.body.err() {
            Some(err) => quote! { #err },
            None => quote! { ::std::convert::Infallible },
        };

        quote! {
            pub fn #ident(&self #args) -> #krate::Reply<#ok, #err> {
                self.inner.call(#id, #val).cast()
            }
        }
    }

    pub fn gen_variant(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let variant = &self.variant;

        let args = self.body.args().map(|args| quote! { #args, });

        let ok = match self.body.ok() {
            Some(ok) => quote! { #ok },
            None => quote! { () },
        };

        let err = match self.body.err() {
            Some(err) => quote! { #err },
            None => quote! { ::std::convert::Infallible },
        };

        quote! {
            #variant(#args #krate::Promise<#ok, #err>),
        }
    }

    pub fn gen_next_call_match_arm(&self, function: &Ident) -> TokenStream {
        let id = &self.id;
        let variant = &self.variant;

        let (args, val) = if self.body.args().is_some() {
            (quote! { args }, Some(quote! { args, }))
        } else {
            (quote! { () }, None)
        };

        quote! {
            #id => match call.deserialize_and_cast() {
                ::std::result::Result::Ok((#args, promise)) => {
                    ::std::task::Poll::Ready(
                        ::std::option::Option::Some(
                            ::std::result::Result::Ok(#function::#variant(#val promise)),
                        ),
                    )
                }

                ::std::result::Result::Err(e) => {
                    ::std::task::Poll::Ready(
                        ::std::option::Option::Some(::std::result::Result::Err(e)),
                    )
                }
            }
        }
    }

    pub fn layout(&self, options: &Options) -> TokenStream {
        let id = &self.id;
        let ident = &self.ident.to_string();
        let krate = options.krate();

        let args = match self.body.args() {
            Some(args) => quote! {
                ::std::option::Option::Some(
                    <#args as #krate::core::introspection::Introspectable>::lexical_id(),
                )
            },

            None => quote! { ::std::option::Option::None },
        };

        let ok = match self.body.ok() {
            Some(ok) => quote! {
                ::std::option::Option::Some(
                    <#ok as #krate::core::introspection::Introspectable>::lexical_id(),
                )
            },

            None => quote! { ::std::option::Option::None },
        };

        let err = match self.body.err() {
            Some(err) => quote! {
                ::std::option::Option::Some(
                    <#err as #krate::core::introspection::Introspectable>::lexical_id(),
                )
            },

            None => quote! { ::std::option::Option::None },
        };

        quote! {
            .function(#id, #ident, #args, #ok, #err)
        }
    }

    pub fn inner_types<'a>(&'a self, inner_types: &mut HashSet<&'a Type>) {
        if let Some(args) = self.body.args() {
            inner_types.insert(args);
        }

        if let Some(ok) = self.body.ok() {
            inner_types.insert(ok);
        }

        if let Some(err) = self.body.err() {
            inner_types.insert(err);
        }
    }
}

impl Parse for FnItem {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![fn]>()?;
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![@]>()?;
        let id = input.parse()?;

        let body = if input.peek(Brace) {
            let content;
            braced!(content in input);
            content.parse()?
        } else {
            input.parse::<Token![;]>()?;
            FnBody::empty()
        };

        let variant = Ident::new_raw(
            &ident.unraw().to_string().to_upper_camel_case(),
            ident.span(),
        );

        Ok(Self {
            ident,
            variant,
            id,
            body,
        })
    }
}
