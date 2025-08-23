use super::{kw, Options};
use crate::doc_string::DocString;
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
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
                pub fn #ident<T>(&self, args: T) -> ::std::result::Result<(), #krate::Error>
                where
                    T: #krate::core::Serialize<#krate::core::tags::As<#ty>>,
                {
                    self.inner.emit_as::<#krate::core::tags::As<#ty>, _>(#id, args)
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

        let ty = self.ty.as_ref().map(|ty| {
            quote! {
                .event_type(<#ty as #krate::core::introspection::Introspectable>::lexical_id())
            }
        });

        quote! {
            .event(
                #krate::core::introspection::ir::EventIr::builder(#id, #name)
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

        let ident_val = format_ident!("r#{}_val", ident);
        let ident_ref = format_ident!("r#{}_ref", ident);

        let subscribe = format_ident!("r#subscribe_{}", ident);
        let unsubscribe = format_ident!("r#unsubscribe_{}", ident);

        let variant = Ident::new_raw(
            &ident.unraw().to_string().to_upper_camel_case(),
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
