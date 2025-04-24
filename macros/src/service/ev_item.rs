use super::{kw, Options};
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, Result, Token, Type};

pub(super) struct EvItem {
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
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

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

    pub fn gen_next_event_match_arm(&self, event: &Ident) -> TokenStream {
        let id = &self.id;
        let variant = &self.variant;

        quote! {
            #id => {
                break ::std::task::Poll::Ready(::std::option::Option::Some(
                    ::std::result::Result::Ok(#event::#variant(ev.cast())),
                ));
            }
        }
    }

    pub fn gen_variant(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let variant = &self.variant;

        let ty = match self.ty {
            Some(ref ty) => quote! { #ty },
            None => quote! { () },
        };

        quote! { #variant(#krate::Event<#ty>), }
    }

    pub fn gen_emitters(&self, options: &Options) -> TokenStream {
        let krate = options.krate();
        let ident = &self.ident;
        let ident_val = &self.ident_val;
        let ident_ref = &self.ident_ref;
        let id = &self.id;

        if let Some(ref ty) = self.ty {
            quote! {
                pub fn #ident<T>(&self, args: T) -> ::std::result::Result<(), #krate::Error>
                where
                    T: #krate::core::Serialize<#krate::core::tags::As<#ty>>,
                {
                    self.inner.emit_as::<#krate::core::tags::As<#ty>, _>(#id, args)
                }

                pub fn #ident_val(&self, args: #ty) -> ::std::result::Result<(), #krate::Error> {
                    self.#ident(args)
                }

                pub fn #ident_ref(&self, args: &#ty) -> ::std::result::Result<(), #krate::Error> {
                    self.#ident(args)
                }
            }
        } else {
            quote! {
                pub fn #ident(&self) -> ::std::result::Result<(), #krate::Error> {
                    self.inner.emit(#id, ())
                }
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

        let ident_val = format_ident!("r#{}_val", ident);
        let ident_ref = format_ident!("r#{}_ref", ident);

        let subscribe = format_ident!("r#subscribe_{}", ident);
        let unsubscribe = format_ident!("r#unsubscribe_{}", ident);

        let variant = Ident::new_raw(
            &ident.unraw().to_string().to_upper_camel_case(),
            ident.span(),
        );

        Ok(Self {
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
