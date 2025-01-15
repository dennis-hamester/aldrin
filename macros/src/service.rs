mod body;
mod ev_fallback_item;
mod ev_item;
mod fn_body;
mod fn_fallback_item;
mod fn_item;
mod item;
mod options;
#[cfg(test)]
mod test;

use body::Body;
use ev_fallback_item::EvFallbackItem;
use ev_item::EvItem;
use fn_body::FnBody;
use fn_fallback_item::FnFallbackItem;
use fn_item::FnItem;
use item::ServiceItem;
use options::Options;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{braced, Ident, Result, Token, Visibility};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(args);
    custom_keyword!(err);
    custom_keyword!(event);
    custom_keyword!(ok);
    custom_keyword!(service);
    custom_keyword!(uuid);
    custom_keyword!(version);
}

pub struct Service {
    options: Options,
    vis: Visibility,
    ident: Ident,
    proxy: Ident,
    event: Ident,
    function: Ident,
    introspection: Ident,
    body: Body,
}

impl Service {
    pub fn generate(&self) -> TokenStream {
        let client = self.options.client().then(|| self.gen_client());
        let server = self.options.server().then(|| self.gen_server());

        let introspection = (self.options.introspection()
            && (self.options.client() || self.options.server()))
        .then(|| self.gen_introspection());

        quote! {
            #client
            #server
            #introspection
        }
    }

    fn gen_client(&self) -> TokenStream {
        let proxy = self.gen_proxy();
        let event = self.gen_event();

        quote! {
            #proxy
            #event
        }
    }

    fn gen_server(&self) -> TokenStream {
        let service = self.gen_service();
        let function = self.gen_function();

        quote! {
            #service
            #function
        }
    }

    fn gen_proxy(&self) -> TokenStream {
        let krate = self.options.krate();
        let vis = &self.vis;
        let proxy = &self.proxy;
        let event = &self.event;
        let body_impl = self.body.gen_proxy(&self.event, &self.options);

        let introspection_if = self.options.introspection_if().map(|feature| {
            quote! { #[cfg(feature = #feature)] }
        });

        let introspection = &self.introspection;
        let introspection = if self.options.introspection() {
            Some(quote! {
                #introspection_if
                #[automatically_derived]
                impl #krate::core::introspection::Introspectable for #proxy {
                    fn layout() -> #krate::core::introspection::Layout {
                        <#introspection as #krate::core::introspection::Introspectable>::layout()
                    }

                    fn lexical_id() -> #krate::core::introspection::LexicalId {
                        <#introspection as #krate::core::introspection::Introspectable>::lexical_id()
                    }

                    fn add_references(references: &mut #krate::core::introspection::References) {
                        <#introspection as #krate::core::introspection::Introspectable>::add_references(
                            references,
                        )
                    }
                }
            })
        } else {
            None
        };

        quote! {
            #[derive(::std::fmt::Debug)]
            #vis struct #proxy {
                #[doc(hidden)]
                inner: #krate::low_level::Proxy,
            }

            impl #proxy {
                #body_impl
            }

            #[automatically_derived]
            impl #krate::private::futures_core::stream::Stream for #proxy {
                type Item = ::std::result::Result<#event, #krate::Error>;

                fn poll_next(
                    mut self: ::std::pin::Pin<&mut Self>,
                    cx: &mut ::std::task::Context,
                ) -> ::std::task::Poll<::std::option::Option<Self::Item>> {
                    self.poll_next_event(cx)
                }
            }

            #[automatically_derived]
            impl #krate::private::futures_core::stream::FusedStream for #proxy {
                fn is_terminated(&self) -> ::std::primitive::bool {
                    self.inner.events_finished()
                }
            }

            #introspection
        }
    }

    fn gen_event(&self) -> TokenStream {
        let vis = &self.vis;
        let vars = self.body.gen_event(&self.options);
        let event = &self.event;

        quote! {
            #[derive(::std::fmt::Debug, ::std::clone::Clone)]
            #vis enum #event {
                #vars
            }
        }
    }

    fn gen_service(&self) -> TokenStream {
        let vis = &self.vis;
        let ident = &self.ident;
        let function = &self.function;
        let krate = self.options.krate();
        let body_impl = self.body.gen_service(&self.function, &self.options);

        let introspection_if = self.options.introspection_if().map(|feature| {
            quote! { #[cfg(feature = #feature)] }
        });

        let introspection = &self.introspection;
        let introspection = if self.options.introspection() {
            Some(quote! {
                #introspection_if
                #[automatically_derived]
                impl #krate::core::introspection::Introspectable for #ident {
                    fn layout() -> #krate::core::introspection::Layout {
                        <#introspection as #krate::core::introspection::Introspectable>::layout()
                    }

                    fn lexical_id() -> #krate::core::introspection::LexicalId {
                        <#introspection as #krate::core::introspection::Introspectable>::lexical_id()
                    }

                    fn add_references(references: &mut #krate::core::introspection::References) {
                        <#introspection as #krate::core::introspection::Introspectable>::add_references(
                            references,
                        )
                    }
                }
            })
        } else {
            None
        };

        quote! {
            #[derive(::std::fmt::Debug)]
            #vis struct #ident {
                #[doc(hidden)]
                inner: #krate::low_level::Service,
            }

            impl #ident {
                #body_impl
            }

            #[automatically_derived]
            impl #krate::private::futures_core::stream::Stream for #ident {
                type Item = ::std::result::Result<#function, #krate::Error>;

                fn poll_next(
                    mut self: ::std::pin::Pin<&mut Self>,
                    cx: &mut ::std::task::Context,
                ) -> ::std::task::Poll<::std::option::Option<Self::Item>> {
                    self.poll_next_call(cx)
                }
            }

            #[automatically_derived]
            impl #krate::private::futures_core::stream::FusedStream for #ident {
                fn is_terminated(&self) -> ::std::primitive::bool {
                    self.inner.is_terminated()
                }
            }

            #introspection
        }
    }

    fn gen_function(&self) -> TokenStream {
        let vis = &self.vis;
        let vars = self.body.gen_function(&self.options);
        let function = &self.function;

        quote! {
            #[derive(::std::fmt::Debug)]
            #vis enum #function {
                #vars
            }
        }
    }

    fn gen_introspection(&self) -> TokenStream {
        let krate = self.options.krate();
        let introspection = &self.introspection;
        let body = self.body.gen_introspection(&self.ident, &self.options);

        let introspection_if = self.options.introspection_if().map(|feature| {
            quote! { #[cfg(feature = #feature)] }
        });

        quote! {
            #introspection_if
            #[doc(hidden)]
            struct #introspection;

            #introspection_if
            #[automatically_derived]
            impl #krate::core::introspection::Introspectable for #introspection {
                #body
            }
        }
    }
}

impl Parse for Service {
    fn parse(input: ParseStream) -> Result<Self> {
        let options = input.parse()?;
        let vis = input.parse()?;
        input.parse::<kw::service>()?;
        let ident = input.parse::<Ident>()?;

        let content;
        braced!(content in input);
        let body = content.parse()?;

        input.parse::<Option<Token![;]>>()?;

        let proxy = Ident::new_raw(&format!("{}Proxy", ident.unraw()), ident.span());
        let event = Ident::new_raw(&format!("{}Event", ident.unraw()), ident.span());
        let function = Ident::new_raw(&format!("{}Function", ident.unraw()), ident.span());
        let introspection =
            Ident::new_raw(&format!("{}Introspection", ident.unraw()), ident.span());

        Ok(Self {
            options,
            vis,
            ident,
            proxy,
            event,
            function,
            introspection,
            body,
        })
    }
}
