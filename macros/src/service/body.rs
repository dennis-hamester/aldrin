use super::{kw, EvFallbackItem, EvItem, FnFallbackItem, FnItem, Options, ServiceItem};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Expr, Ident, Result, Token};

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

        let introspection_fns = options.introspection().then(|| {
            quote! {
                #introspection_if
                pub fn introspection() -> #krate::core::introspection::Introspection {
                    #krate::core::introspection::Introspection::new::<Self>()
                }

                #introspection_if
                pub async fn query_introspection(
                    &self,
                ) -> ::std::result::Result<
                    ::std::option::Option<#krate::core::introspection::Introspection>,
                    #krate::Error,
                > {
                    self.inner.query_introspection().await
                }
            }
        });

        let fn_calls = self
            .items
            .iter()
            .filter_map(ServiceItem::as_function)
            .map(|func| func.gen_calls(options))
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

        let next_event_fallback = match self.event_fallback() {
            Some(fallback) => fallback.gen_next_event_match_arm(event),
            None => quote! { _ => {} },
        };

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

            #introspection_fns
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
                        #next_event_fallback
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

    pub fn gen_event(&self, options: &Options) -> TokenStream {
        let mut variants = self
            .items
            .iter()
            .filter_map(ServiceItem::as_event)
            .map(|ev| ev.gen_variant(options))
            .collect::<TokenStream>();

        if let Some(fallback) = self.event_fallback() {
            variants.extend(fallback.gen_variant());
        }

        variants
    }

    pub fn gen_event_impl(&self) -> TokenStream {
        let variants = |f| {
            self.items
                .iter()
                .filter_map(ServiceItem::as_event)
                .map(EvItem::variant)
                .chain(self.event_fallback().map(EvFallbackItem::variant))
                .map(move |ev| quote! { Self::#ev(ref event) => event.#f(), })
        };

        let id = variants(quote!(id));
        let timestamp = variants(quote!(timestamp));

        quote! {
            pub fn id(&self) -> ::std::primitive::u32 {
                match *self {
                    #( #id )*
                }
            }

            pub fn timestamp(&self) -> ::std::time::Instant {
                match *self {
                    #( #timestamp )*
                }
            }
        }
    }

    pub fn gen_service(&self, function: &Ident, options: &Options) -> TokenStream {
        let uuid = &self.uuid;
        let version = &self.version;
        let krate = options.krate();

        let introspection_if = options.introspection_if().map(|feature| {
            quote! { #[cfg(feature = #feature)] }
        });

        let info_type_id = options.introspection().then(|| {
            quote! {
                #introspection_if
                let info = info.set_type_id(#krate::core::TypeId::compute::<Self>());
            }
        });

        let introspection_fns = options.introspection().then(|| {
            quote! {
                #introspection_if
                pub fn introspection() -> #krate::core::introspection::Introspection {
                    #krate::core::introspection::Introspection::new::<Self>()
                }

                #introspection_if
                pub async fn query_introspection(
                    &self,
                ) -> ::std::result::Result<
                    ::std::option::Option<#krate::core::introspection::Introspection>,
                    #krate::Error,
                > {
                    self.inner.query_introspection().await
                }
            }
        });

        let ev_emitters = self
            .items
            .iter()
            .filter_map(ServiceItem::as_event)
            .map(|ev| ev.gen_emitters(options))
            .collect::<TokenStream>();

        let next_call_match_arms = self
            .items
            .iter()
            .filter_map(ServiceItem::as_function)
            .map(|func| func.gen_next_call_match_arm(function))
            .collect::<TokenStream>();

        let next_call_fallback = match self.function_fallback() {
            Some(fallback) => fallback.gen_next_call_match_arm(function),

            None => quote! {
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
            },
        };

        quote! {
            pub const UUID: #krate::core::ServiceUuid = #uuid;
            pub const VERSION: ::std::primitive::u32 = #version;

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

            #introspection_fns

            pub fn client(&self) -> &#krate::Handle {
                self.inner.client()
            }

            pub async fn destroy(&self) -> ::std::result::Result<(), #krate::Error> {
                self.inner.destroy().await
            }

            #ev_emitters

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
                    #next_call_fallback
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
        let mut variants = self
            .items
            .iter()
            .filter_map(ServiceItem::as_function)
            .map(|func| func.gen_variant(options))
            .collect::<TokenStream>();

        if let Some(fallback) = self.function_fallback() {
            variants.extend(fallback.gen_variant());
        }

        variants
    }

    pub fn gen_function_impl(&self, options: &Options) -> TokenStream {
        let krate = options.krate();

        let variants = |f, args, borrow, wait| {
            self.items
                .iter()
                .filter_map(ServiceItem::as_function)
                .map(FnItem::variant)
                .chain(self.function_fallback().map(FnFallbackItem::variant))
                .map(move |func| quote! { Self::#func(#borrow call) => call.#f(#args) #wait, })
        };

        let client = variants(quote!(client), quote!(), quote!(ref), quote!());
        let id = variants(quote!(id), quote!(), quote!(ref), quote!());
        let version = variants(quote!(version), quote!(), quote!(ref), quote!());
        let timestamp = variants(quote!(timestamp), quote!(), quote!(ref), quote!());
        let abort = variants(quote!(abort), quote!(), quote!(), quote!());
        let is_aborted = variants(quote!(is_aborted), quote!(), quote!(ref mut), quote!());
        let poll_aborted = variants(quote!(poll_aborted), quote!(cx), quote!(ref mut), quote!());
        let aborted = variants(quote!(aborted), quote!(), quote!(ref mut), quote!(.await));

        quote! {
            pub fn client(&self) -> &#krate::Handle {
                match *self {
                    #( #client )*
                }
            }

            pub fn id(&self) -> ::std::primitive::u32 {
                match *self {
                    #( #id )*
                }
            }

            pub fn version(&self) -> ::std::option::Option<::std::primitive::u32> {
                match *self {
                    #( #version )*
                }
            }

            pub fn timestamp(&self) -> ::std::time::Instant {
                match *self {
                    #( #timestamp )*
                }
            }

            pub fn abort(self) -> ::std::result::Result<(), #krate::Error> {
                match self {
                    #( #abort )*
                }
            }

            pub fn is_abort(&mut self) -> ::std::primitive::bool {
                match *self {
                    #( #is_aborted )*
                }
            }

            pub fn poll_aborted(&mut self, cx: &mut ::std::task::Context) -> ::std::task::Poll<()> {
                match *self {
                    #( #poll_aborted )*
                }
            }

            pub async fn aborted(&mut self) {
                match *self {
                    #( #aborted )*
                }
            }
        }
    }

    pub fn gen_introspection(&self, service: &Ident, options: &Options) -> TokenStream {
        let krate = options.krate();
        let schema = options.schema().unwrap();
        let service = service.unraw().to_string();
        let uuid = &self.uuid;
        let version = &self.version;

        let items = self
            .items
            .iter()
            .map(|item| item.layout(options))
            .collect::<TokenStream>();

        let mut references = HashSet::new();
        for item in &self.items {
            item.add_references(&mut references);
        }
        let references_len = references.len();
        let references = references.into_iter();

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

            fn add_references(references: &mut #krate::core::introspection::References) {
                let types: [#krate::core::introspection::DynIntrospectable; #references_len] = [
                    #( #krate::core::introspection::DynIntrospectable::new::<#references>(), )*
                ];

                references.extend(types);
            }
        }
    }

    fn function_fallback(&self) -> Option<&FnFallbackItem> {
        self.items
            .iter()
            .find_map(ServiceItem::as_fallback_function)
    }

    fn event_fallback(&self) -> Option<&EvFallbackItem> {
        self.items.iter().find_map(ServiceItem::as_fallback_event)
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

        let mut ev_fallback = false;
        let mut fn_fallback = false;
        for item in &items {
            match item {
                ServiceItem::Event(ev) => {
                    if ev_fallback || fn_fallback {
                        return Err(Error::new_spanned(
                            ev.ident(),
                            "events must be defined before any fallbacks",
                        ));
                    }
                }

                ServiceItem::EventFallback(ev) => {
                    if ev_fallback {
                        return Err(Error::new_spanned(
                            ev.ident(),
                            "there can be at most one evnt fallback",
                        ));
                    } else {
                        ev_fallback = true;
                    }
                }

                ServiceItem::Function(func) => {
                    if ev_fallback || fn_fallback {
                        return Err(Error::new_spanned(
                            func.ident(),
                            "functions must be defined before any fallbacks",
                        ));
                    }
                }

                ServiceItem::FunctionFallback(func) => {
                    if fn_fallback {
                        return Err(Error::new_spanned(
                            func.ident(),
                            "there can be at most one function fallback",
                        ));
                    } else {
                        fn_fallback = true;
                    }
                }
            }
        }

        Ok(Self {
            uuid,
            version,
            items,
        })
    }
}
