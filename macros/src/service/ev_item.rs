use super::{kw, Options};
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, Result, Token, Type};

pub(super) struct EvItem {
    ident: Ident,
    subscribe: Ident,
    unsubscribe: Ident,
    variant: Ident,
    id: LitInt,
    ty: Option<Type>,
}

impl EvItem {
    pub fn gen_subscribe_call(&self) -> TokenStream {
        let subscribe = &self.subscribe;
        quote! { self.#subscribe().await?; }
    }

    pub fn gen_subscribe_fn(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let subscribe = &self.subscribe;
        let id = &self.id;

        quote! {
            pub async fn #subscribe(&self) -> ::std::result::Result<(), #krate::Error> {
                self.inner.subscribe(#id).await
            }
        }
    }

    pub fn gen_unsubscribe_fn(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let unsubscribe = &self.unsubscribe;
        let id = &self.id;

        quote! {
            pub async fn #unsubscribe(&self) -> ::std::result::Result<(), #krate::Error> {
                self.inner.unsubscribe(#id).await
            }
        }
    }

    pub fn gen_next_event_match_arm(&self, event: &Ident, options: &Options) -> TokenStream {
        let krate = options.krate();
        let id = &self.id;
        let variant = &self.variant;

        let ok = if self.ty.is_some() {
            quote! {
                ::std::result::Result::Ok(val) => {
                    break ::std::task::Poll::Ready(
                        ::std::option::Option::Some(
                            ::std::result::Result::Ok(#event::#variant(val)),
                        ),
                    );
                }
            }
        } else {
            quote! {
                ::std::result::Result::Ok(()) => {
                    break ::std::task::Poll::Ready(
                        ::std::option::Option::Some(
                            ::std::result::Result::Ok(#event::#variant),
                        ),
                    );
                }
            }
        };

        quote! {
            #id => match ev.deserialize() {
                #ok

                ::std::result::Result::Err(e) => {
                    break ::std::task::Poll::Ready(
                        ::std::option::Option::Some(
                            ::std::result::Result::Err(
                                #krate::Error::invalid_arguments(
                                    ev.id(),
                                    ::std::option::Option::Some(e),
                                ),
                            ),
                        ),
                    );
                }
            }
        }
    }

    pub fn gen_variant(&self) -> TokenStream {
        let variant = &self.variant;

        if let Some(ref ty) = self.ty {
            quote! { #variant(#ty), }
        } else {
            quote! { #variant, }
        }
    }

    pub fn gen_emit(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let ident = &self.ident;
        let id = &self.id;

        let (args, val) = match self.ty {
            Some(ref ty) => (
                Some(quote! { , args: #krate::core::SerializeArg<'_, #ty> }),
                quote! { &args },
            ),

            None => (None, quote! { &() }),
        };

        quote! {
            pub fn #ident(&self #args) -> ::std::result::Result<(), #krate::Error> {
                self.inner.emit(#id, #val)
            }
        }
    }

    pub fn layout(&self, options: &Options) -> TokenStream {
        let id = &self.id;
        let name = self.ident.unraw().to_string();
        let krate = options.krate();

        let ty = match self.ty {
            Some(ref ty) => quote! {
                ::std::option::Option::Some(
                    <#ty as #krate::core::introspection::Introspectable>::lexical_id(),
                )
            },

            None => quote! { ::std::option::Option::None },
        };

        quote! {
            .event(#id, #name, #ty)
        }
    }

    pub fn add_references<'a>(&'a self, references: &mut HashSet<&'a Type>) {
        if let Some(ref ty) = self.ty {
            references.insert(ty);
        }
    }
}

impl Parse for EvItem {
    fn parse(input: ParseStream) -> Result<Self> {
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

        let subscribe = Ident::new_raw(&format!("subscribe_{}", ident.unraw()), ident.span());
        let unsubscribe = Ident::new_raw(&format!("unsubscribe_{}", ident.unraw()), ident.span());

        let variant = Ident::new_raw(
            &ident.unraw().to_string().to_upper_camel_case(),
            ident.span(),
        );

        Ok(Self {
            ident,
            subscribe,
            unsubscribe,
            variant,
            id,
            ty,
        })
    }
}
