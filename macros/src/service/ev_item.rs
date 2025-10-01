use super::{kw, Options};
use crate::doc_string::DocString;
use aldrin_codegen::rust::names;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, Result, Token, Type};

pub(super) struct EvItem {
    doc: DocString,
    ident: Ident,
    ident_val: Ident,
    ident_ref: Ident,
    subscribe: Ident,
    unsubscribe: Ident,
    variant: Ident,
    id: LitInt,
    ty: Option<Type>,
}

impl EvItem {
    pub(crate) fn ident(&self) -> &Ident {
        &self.ident
    }

    pub(crate) fn variant(&self) -> &Ident {
        &self.variant
    }

    pub(crate) fn gen_subscribe_call(&self) -> TokenStream {
        let subscribe = &self.subscribe;
        quote! { self.#subscribe().await?; }
    }

    pub(crate) fn gen_subscribe_fn(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let subscribe = &self.subscribe;
        let id = &self.id;
        let doc = &self.doc;

        quote! {
            #doc
            pub async fn #subscribe(&self) -> ::std::result::Result<(), #krate::Error> {
                self.inner.subscribe(#id).await
            }
        }
    }

    pub(crate) fn gen_unsubscribe_fn(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let unsubscribe = &self.unsubscribe;
        let id = &self.id;
        let doc = &self.doc;

        quote! {
            #doc
            pub async fn #unsubscribe(&self) -> ::std::result::Result<(), #krate::Error> {
                self.inner.unsubscribe(#id).await
            }
        }
    }

    pub(crate) fn gen_next_event_match_arm(&self, event: &Ident, options: &Options) -> TokenStream {
        let krate = options.krate();
        let id = &self.id;
        let variant = &self.variant;

        quote! {
            #id => {
                break match ev.deserialize_and_cast() {
                    ::std::result::Result::Ok(ev) => ::std::task::Poll::Ready(
                        ::std::option::Option::Some(::std::result::Result::Ok(#event::#variant(ev))),
                    ),

                    ::std::result::Result::Err(e) => {
                        ::std::task::Poll::Ready(::std::option::Option::Some(
                            ::std::result::Result::Err(#krate::Error::invalid_arguments(
                                ev.id(),
                                ::std::option::Option::Some(e),
                            )),
                        ))
                    }
                };
            }
        }
    }

    pub(crate) fn gen_variant(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let variant = &self.variant;
        let doc = &self.doc;

        let ty = match self.ty {
            Some(ref ty) => quote! { #ty },
            None => quote! { () },
        };

        quote! {
            #doc
            #variant(#krate::Event<#ty>),
        }
    }

    pub(crate) fn gen_emitters(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let ident = &self.ident;
        let ident_val = &self.ident_val;
        let ident_ref = &self.ident_ref;
        let id = &self.id;
        let doc = &self.doc;

        if let Some(ref ty) = self.ty {
            quote! {
                #doc
                pub fn #ident(
                    &self,
                    args: impl #krate::core::Serialize<#krate::core::tags::As<#ty>>,
                ) -> ::std::result::Result<(), #krate::Error> {
                    self.inner.emit_as::<#krate::core::tags::As<#ty>>(#id, args)
                }

                #doc
                pub fn #ident_val(&self, args: #ty) -> ::std::result::Result<(), #krate::Error> {
                    self.#ident(args)
                }

                #doc
                pub fn #ident_ref(&self, args: &#ty) -> ::std::result::Result<(), #krate::Error> {
                    self.#ident(args)
                }
            }
        } else {
            quote! {
                #doc
                pub fn #ident(&self) -> ::std::result::Result<(), #krate::Error> {
                    self.inner.emit(#id, ())
                }
            }
        }
    }

    pub(crate) fn layout(&self, options: &Options) -> TokenStream {
        let id = &self.id;
        let name = self.ident.unraw().to_string();
        let krate = options.krate();
        let doc = self.doc.to_introspection();

        let ty = self.ty.as_ref().map(|ty| {
            quote! {
                .event_type(<#ty as #krate::core::introspection::Introspectable>::lexical_id())
            }
        });

        quote! {
            .event(
                #krate::core::introspection::ir::EventIr::builder(#id, #name)
                    #doc
                    #ty
                    .finish(),
            )
        }
    }

    pub(crate) fn add_references<'a>(&'a self, references: &mut HashSet<&'a Type>) {
        if let Some(ref ty) = self.ty {
            references.insert(ty);
        }
    }
}

impl Parse for EvItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let doc = input.parse()?;
        input.parse::<kw::event>()?;
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![@]>()?;
        let id = input.parse()?;

        let ty = if input.parse::<Token![=]>().is_ok() {
            input.parse().map(Some)?
        } else {
            None
        };

        input.parse::<Token![;]>()?;

        let ident_val = Ident::new_raw(&names::emit_val(&ident.unraw().to_string()), ident.span());
        let ident_ref = Ident::new_raw(&names::emit_ref(&ident.unraw().to_string()), ident.span());
        let subscribe = Ident::new_raw(&names::subscribe(&ident.unraw().to_string()), ident.span());

        let unsubscribe = Ident::new_raw(
            &names::unsubscribe(&ident.unraw().to_string()),
            ident.span(),
        );

        let variant = Ident::new_raw(
            &names::event_variant(&ident.unraw().to_string()),
            ident.span(),
        );

        Ok(Self {
            doc,
            ident,
            ident_val,
            ident_ref,
            subscribe,
            unsubscribe,
            variant,
            id,
            ty,
        })
    }
}
