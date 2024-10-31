use super::{kw, EvItem, Options, ServiceItem};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, Result, Token};

pub(super) struct Body {
    uuid: Expr,
    version: Expr,
    items: Vec<ServiceItem>,
}

impl Body {
    pub fn gen_proxy(&self, event: &Ident, options: &Options) -> TokenStream {
        let krate = options.krate();
        let uuid = &self.uuid;
        let version = &self.version;

        let introspection_if = options.introspection_if().map(|feature| {
            quote! { #[cfg(feature = #feature)] }
        });

        let query_introspection = if options.introspection() {
            Some(quote! {
                #introspection_if
                pub async fn query_introspection(
                    &self,
                ) -> ::std::result::Result<
                    ::std::option::Option<#krate::core::introspection::Introspection>,
                    #krate::Error,
                > {
                    self.inner.query_introspection().await
                }
            })
        } else {
            None
        };

        let fn_calls = self
            .items
            .iter()
            .filter_map(ServiceItem::as_function)
            .map(|func| func.gen_call(options))
            .collect::<TokenStream>();

        let subscribe_all_body = self
            .items
            .iter()
            .filter_map(ServiceItem::as_event)
            .map(EvItem::gen_subscribe_call)
            .collect::<TokenStream>();

        let subscribe_fns = self
            .items
            .iter()
            .filter_map(ServiceItem::as_event)
            .map(|ev| ev.gen_subscribe_fn(options))
            .collect::<TokenStream>();

        let unsubscribe_fns = self
            .items
            .iter()
            .filter_map(ServiceItem::as_event)
            .map(|ev| ev.gen_unsubscribe_fn(options))
            .collect::<TokenStream>();

        let next_event_match_arms = self
            .items
            .iter()
            .filter_map(ServiceItem::as_event)
            .map(|ev| ev.gen_next_event_match_arm(event, options))
            .collect::<TokenStream>();

        quote! {
            pub const UUID: #krate::core::ServiceUuid = #uuid;
            pub const VERSION: ::std::primitive::u32 = #version;

            pub async fn new(
                client: &#krate::Handle,
                id: #krate::core::ServiceId,
            ) -> ::std::result::Result<Self, #krate::Error>
            {
                if id.uuid != Self::UUID {
                    return ::std::result::Result::Err(#krate::Error::InvalidService);
                }

                let inner = #krate::low_level::Proxy::new(client, id).await?;
                Ok(Self { inner })
            }

            pub fn inner(&self) -> &#krate::low_level::Proxy {
                &self.inner
            }

            pub fn inner_mut(&mut self) -> &mut #krate::low_level::Proxy {
                &mut self.inner
            }

            pub fn into_inner(self) -> #krate::low_level::Proxy {
                self.inner
            }

            pub fn client(&self) -> &#krate::Handle {
                self.inner.client()
            }

            pub fn id(&self) -> #krate::core::ServiceId {
                self.inner.id()
            }

            pub fn version(&self) -> ::std::primitive::u32 {
                self.inner.version()
            }

            pub fn type_id(&self) -> ::std::option::Option<#krate::core::TypeId> {
                self.inner.type_id()
            }

            #query_introspection
            #fn_calls

            pub async fn subscribe_all(&self) -> ::std::result::Result<(), #krate::Error> {
                #subscribe_all_body
                Ok(())
            }

            pub async fn unsubscribe_all(&self) -> ::std::result::Result<(), #krate::Error> {
                self.inner.unsubscribe_all().await
            }

            #subscribe_fns
            #unsubscribe_fns

            pub fn poll_next_event(
                &mut self,
                cx: &mut ::std::task::Context,
            ) -> ::std::task::Poll<
                ::std::option::Option<::std::result::Result<#event, #krate::Error>>,
            > {
                loop {
                    let ev = match self.inner.poll_next_event(cx) {
                        ::std::task::Poll::Ready(::std::option::Option::Some(ev)) => ev,

                        ::std::task::Poll::Ready(::std::option::Option::None) => {
                            break ::std::task::Poll::Ready(None);
                        }

                        ::std::task::Poll::Pending => break ::std::task::Poll::Pending,
                    };

                    match ev.id() {
                        #next_event_match_arms
                        _ => {}
                    }
                }
            }

            pub async fn next_event(
                &mut self,
            ) -> ::std::option::Option<::std::result::Result<#event, #krate::Error>> {
                ::std::future::poll_fn(|cx| self.poll_next_event(cx)).await
            }
        }
    }

    pub fn gen_event(&self) -> TokenStream {
        self.items
            .iter()
            .filter_map(ServiceItem::as_event)
            .map(EvItem::gen_variant)
            .collect::<TokenStream>()
    }

    pub fn gen_service(&self, function: &Ident, options: &Options) -> TokenStream {
        let uuid = &self.uuid;
        let version = &self.version;
        let krate = options.krate();

        let introspection_if = options.introspection_if().map(|feature| {
            quote! { #[cfg(feature = #feature)] }
        });

        let info_type_id = if options.introspection() {
            Some(quote! {
                #introspection_if
                let info = info.set_type_id(#krate::core::TypeId::compute::<Self>());
            })
        } else {
            None
        };

        let query_introspection = if options.introspection() {
            Some(quote! {
                #introspection_if
                pub async fn query_introspection(
                    &self,
                ) -> ::std::result::Result<
                    ::std::option::Option<#krate::core::introspection::Introspection>,
                    #krate::Error,
                > {
                    self.inner.query_introspection().await
                }
            })
        } else {
            None
        };

        let ev_emiters = self
            .items
            .iter()
            .filter_map(ServiceItem::as_event)
            .map(|ev| ev.gen_emit(options))
            .collect::<TokenStream>();

        let next_call_match_arms = self
            .items
            .iter()
            .filter_map(ServiceItem::as_function)
            .map(|func| func.gen_next_call_match_arm(function))
            .collect::<TokenStream>();

        quote! {
            const UUID: #krate::core::ServiceUuid = #uuid;
            const VERSION: ::std::primitive::u32 = #version;

            pub async fn new(object: &#krate::Object) -> ::std::result::Result<Self, #krate::Error> {
                let info = #krate::low_level::ServiceInfo::new(Self::VERSION);
                #info_type_id

                let inner = object.create_service(Self::UUID, info).await?;
                ::std::result::Result::Ok(Self { inner })
            }

            pub fn inner(&self) -> &#krate::low_level::Service {
                &self.inner
            }

            pub fn inner_mut(&mut self) -> &mut #krate::low_level::Service {
                &mut self.inner
            }

            pub fn into_inner(self) -> #krate::low_level::Service {
                self.inner
            }

            pub fn id(&self) -> #krate::core::ServiceId {
                self.inner.id()
            }

            pub fn version(&self) -> ::std::primitive::u32 {
                self.inner.version()
            }

            pub fn type_id(&self) -> ::std::option::Option<#krate::core::TypeId> {
                self.inner.type_id()
            }

            #query_introspection

            pub fn client(&self) -> &#krate::Handle {
                self.inner.client()
            }

            pub async fn destroy(&self) -> ::std::result::Result<(), #krate::Error> {
                self.inner.destroy().await
            }

            #ev_emiters

            pub fn poll_next_call(
                &mut self,
                cx: &mut ::std::task::Context,
            ) -> ::std::task::Poll<
                ::std::option::Option<::std::result::Result<#function, #krate::Error>>,
            > {
                let call = match self.inner.poll_next_call(cx) {
                    ::std::task::Poll::Ready(::std::option::Option::Some(call)) => call,

                    ::std::task::Poll::Ready(::std::option::Option::None) => {
                        return ::std::task::Poll::Ready(::std::option::Option::None);
                    }

                    ::std::task::Poll::Pending => return ::std::task::Poll::Pending,
                };

                match call.id() {
                    #next_call_match_arms

                    id => {
                        let _ = call.into_promise().invalid_function();

                        ::std::task::Poll::Ready(
                            ::std::option::Option::Some(
                                ::std::result::Result::Err(
                                    #krate::Error::invalid_function(id),
                                ),
                            ),
                        )
                    }
                }
            }

            pub async fn next_call(
                &mut self,
            ) -> ::std::option::Option<::std::result::Result<#function, #krate::Error>> {
                ::std::future::poll_fn(|cx| self.poll_next_call(cx)).await
            }
        }
    }

    pub fn gen_function(&self, options: &Options) -> TokenStream {
        self.items
            .iter()
            .filter_map(ServiceItem::as_function)
            .map(|func| func.gen_variant(options))
            .collect::<TokenStream>()
    }

    pub fn gen_introspection(&self, service: &Ident, options: &Options) -> TokenStream {
        let krate = options.krate();
        let schema = options.schema().unwrap();
        let service = service.to_string();
        let uuid = &self.uuid;
        let version = &self.version;

        let items = self
            .items
            .iter()
            .map(|item| item.layout(options))
            .collect::<TokenStream>();

        let mut inner_types = HashSet::new();
        for item in &self.items {
            item.inner_types(&mut inner_types);
        }
        let inner_types_len = inner_types.len();
        let inner_types = inner_types.into_iter();

        quote! {
            fn layout() -> #krate::core::introspection::Layout {
                #krate::core::introspection::Layout::Service(
                    #krate::core::introspection::Service::builder(
                        #schema,
                        #service,
                        #uuid,
                        #version,
                    )
                    #items
                    .finish(),
                )
            }

            fn lexical_id() -> #krate::core::introspection::LexicalId {
                #krate::core::introspection::LexicalId::service(#schema, #service)
            }

            fn inner_types(
                types: &mut ::std::vec::Vec<#krate::core::introspection::DynIntrospectable>,
            ) {
                let inner_types: [#krate::core::introspection::DynIntrospectable; #inner_types_len] = [
                    #( #krate::core::introspection::DynIntrospectable::new::<#inner_types>(), )*
                ];

                types.extend(inner_types);
            }
        }
    }
}

impl Parse for Body {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::uuid>()?;
        input.parse::<Token![=]>()?;
        let uuid = input.parse()?;
        input.parse::<Token![;]>()?;

        input.parse::<kw::version>()?;
        input.parse::<Token![=]>()?;
        let version = input.parse()?;
        input.parse::<Token![;]>()?;

        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }

        Ok(Self {
            uuid,
            version,
            items,
        })
    }
}
