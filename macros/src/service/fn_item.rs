use super::{FnBody, Options};
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, Ident, LitInt, Result, Token, Type};

pub(super) struct FnItem {
    ident: Ident,
    ident_val: Ident,
    ident_ref: Ident,
    variant: Ident,
    id: LitInt,
    body: FnBody,
}

impl FnItem {
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn variant(&self) -> &Ident {
        &self.variant
    }

    pub fn gen_calls(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let ident = &self.ident;
        let ident_val = &self.ident_val;
        let ident_ref = &self.ident_ref;
        let id = &self.id;

        let ok = match self.body.ok() {
            Some(ok) => quote! { #ok },
            None => quote! { () },
        };

        let err = match self.body.err() {
            Some(err) => quote! { #err },
            None => quote! { ::std::convert::Infallible },
        };

        if let Some(args) = self.body.args() {
            quote! {
                pub fn #ident<T>(&self, args: T) -> #krate::PendingReply<#ok, #err>
                where
                    T: #krate::core::Serialize<#krate::core::tags::As<#args>>,
                {
                    self.inner
                        .call_as::<#krate::core::tags::As<#args>, _>(
                            #id,
                            args,
                            ::std::option::Option::Some(Self::VERSION),
                        ).cast()
                }

                pub fn #ident_val(&self, args: #args) -> #krate::PendingReply<#ok, #err> {
                    self.#ident(args)
                }

                pub fn #ident_ref(&self, args: &#args) -> #krate::PendingReply<#ok, #err> {
                    self.#ident(args)
                }
            }
        } else {
            quote! {
                pub fn #ident(&self) -> #krate::PendingReply<#ok, #err> {
                    self.inner.call(#id, (), ::std::option::Option::Some(Self::VERSION)).cast()
                }
            }
        }
    }

    pub fn gen_variant(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let variant = &self.variant;

        let args = match self.body.args() {
            Some(args) => quote! { #args },
            None => quote! { () },
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
            #variant(#krate::Call<#args, #ok, #err>),
        }
    }

    pub fn gen_next_call_match_arm(&self, call: &Ident) -> TokenStream {
        let id = &self.id;
        let variant = &self.variant;

        quote! {
            #id => match call.deserialize_and_cast() {
                ::std::result::Result::Ok((call)) => {
                    ::std::task::Poll::Ready(
                        ::std::option::Option::Some(
                            ::std::result::Result::Ok(#call::#variant(call)),
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
        let name = self.ident.unraw().to_string();
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
            .function(#id, #name, #args, #ok, #err)
        }
    }

    pub fn add_references<'a>(&'a self, references: &mut HashSet<&'a Type>) {
        if let Some(args) = self.body.args() {
            references.insert(args);
        }

        if let Some(ok) = self.body.ok() {
            references.insert(ok);
        }

        if let Some(err) = self.body.err() {
            references.insert(err);
        }
    }
}

impl Parse for FnItem {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![fn]>()?;
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![@]>()?;
        let id = input.parse()?;

        let lookahead = input.lookahead1();
        let body = if lookahead.peek(Brace) {
            let content;
            braced!(content in input);
            content.parse()?
        } else if lookahead.peek(Token![=]) {
            let body = input.call(FnBody::parse_simplified)?;
            input.parse::<Token![;]>()?;
            body
        } else if lookahead.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            FnBody::empty()
        } else {
            return Err(lookahead.error());
        };

        let ident_val = format_ident!("r#{}_val", ident);
        let ident_ref = format_ident!("r#{}_ref", ident);

        let variant = Ident::new_raw(
            &ident.unraw().to_string().to_upper_camel_case(),
            ident.span(),
        );

        Ok(Self {
            ident,
            ident_val,
            ident_ref,
            variant,
            id,
            body,
        })
    }
}
