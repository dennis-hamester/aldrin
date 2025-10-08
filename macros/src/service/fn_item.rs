use super::{FnBody, ItemOptions, Options};
use aldrin_codegen::rust::names;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, Ident, LitInt, Result, Token, Type};

pub(super) struct FnItem {
    options: ItemOptions,
    ident: Ident,
    ident_val: Ident,
    ident_ref: Ident,
    variant: Ident,
    id: LitInt,
    body: FnBody,
}

impl FnItem {
    pub(crate) fn ident(&self) -> &Ident {
        &self.ident
    }

    pub(crate) fn variant(&self) -> &Ident {
        &self.variant
    }

    pub(crate) fn gen_calls(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let ident = &self.ident;
        let ident_val = &self.ident_val;
        let ident_ref = &self.ident_ref;
        let id = &self.id;
        let doc = &self.options.doc();

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
                #doc
                pub fn #ident(
                    &self,
                    args: impl #krate::core::Serialize<#krate::core::tags::As<#args>>,
                ) -> #krate::PendingReply<#ok, #err> {
                    self.inner
                        .call_as::<#krate::core::tags::As<#args>>(
                            #id,
                            args,
                            ::std::option::Option::Some(Self::VERSION),
                        ).cast()
                }

                #doc
                pub fn #ident_val(&self, args: #args) -> #krate::PendingReply<#ok, #err> {
                    self.#ident(args)
                }

                #doc
                pub fn #ident_ref(&self, args: &#args) -> #krate::PendingReply<#ok, #err> {
                    self.#ident(args)
                }
            }
        } else {
            quote! {
                #doc
                pub fn #ident(&self) -> #krate::PendingReply<#ok, #err> {
                    self.inner.call(#id, (), ::std::option::Option::Some(Self::VERSION)).cast()
                }
            }
        }
    }

    pub(crate) fn gen_variant(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let variant = &self.variant;
        let doc = &self.options.doc();

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
            #doc
            #variant(#krate::Call<#args, #ok, #err>),
        }
    }

    pub(crate) fn gen_next_call_match_arm(&self, call: &Ident) -> TokenStream {
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

    pub(crate) fn layout(&self, options: &Options) -> TokenStream {
        let id = &self.id;
        let name = self.ident.unraw().to_string();
        let krate = options.krate();
        let doc = self.options.doc_alt().to_introspection();

        let args = self.body.args().map(|args| {
            quote! {
                .args(<#args as #krate::core::introspection::Introspectable>::lexical_id())
            }
        });

        let ok = self.body.ok().map(|ok| {
            quote! {
                .ok(<#ok as #krate::core::introspection::Introspectable>::lexical_id())
            }
        });

        let err = self.body.err().map(|err| {
            quote! {
                .err(<#err as #krate::core::introspection::Introspectable>::lexical_id())
            }
        });

        quote! {
            .function(
                #krate::core::introspection::ir::FunctionIr::builder(#id, #name)
                    #doc
                    #args
                    #ok
                    #err
                    .finish(),
            )
        }
    }

    pub(crate) fn add_references<'a>(&'a self, references: &mut HashSet<&'a Type>) {
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
        let options = input.parse()?;
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

        let ident_val = Ident::new_raw(&names::call_val(&ident.unraw().to_string()), ident.span());
        let ident_ref = Ident::new_raw(&names::call_ref(&ident.unraw().to_string()), ident.span());

        let variant = Ident::new_raw(
            &names::function_variant(&ident.unraw().to_string()),
            ident.span(),
        );

        Ok(Self {
            options,
            ident,
            ident_val,
            ident_ref,
            variant,
            id,
            body,
        })
    }
}
