use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Error, Expr, ExprLit, Lit, LitStr, Meta, MetaNameValue, Result};

pub(crate) struct DocString {
    inner: Vec<LitStr>,
}

impl DocString {
    pub(crate) fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub(crate) fn push(&mut self, doc: LitStr) {
        self.inner.push(doc);
    }

    pub(crate) fn push_attr(&mut self, attr: Attribute) -> Result<()> {
        if !attr.path().is_ident("doc") {
            return Err(Error::new_spanned(attr, "extected attribute `doc`"));
        }

        if let Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit {
                lit: Lit::Str(doc), ..
            }),
            ..
        }) = attr.meta
        {
            self.inner.push(doc);
            Ok(())
        } else {
            Err(Error::new_spanned(attr, "only doc comments are supported"))
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub(crate) fn to_introspection(&self) -> Option<TokenStream> {
        let mut doc_string = String::new();

        for doc in &self.inner {
            let doc = doc.value();

            if doc.is_empty() {
                doc_string.push('\n');
                continue;
            }

            for line in doc.lines() {
                let line = line.strip_prefix(' ').unwrap_or(line).trim_end();

                doc_string.push_str(line);
                doc_string.push('\n');
            }
        }

        if doc_string.is_empty() {
            None
        } else {
            Some(quote! { .doc(#doc_string) })
        }
    }
}

impl Parse for DocString {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut doc = Self::new();

        let attrs = input.call(Attribute::parse_outer)?;

        for attr in attrs {
            doc.push_attr(attr)?;
        }

        Ok(doc)
    }
}

impl ToTokens for DocString {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.inner.iter().map(|doc| quote!(#[doc = #doc])));
    }
}
