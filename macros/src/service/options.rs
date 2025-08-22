use crate::doc_string::DocString;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Attribute, Error, LitBool, LitStr, Path, Result};

pub(super) struct Options {
    doc: DocString,
    krate: Path,
    client: bool,
    server: bool,
    introspection: bool,
    introspection_if: Option<LitStr>,
    schema: Option<LitStr>,
}

impl Options {
    pub(crate) fn doc(&self) -> &DocString {
        &self.doc
    }

    pub(crate) fn krate(&self) -> &Path {
        &self.krate
    }

    pub(crate) fn client(&self) -> bool {
        self.client
    }

    pub(crate) fn server(&self) -> bool {
        self.server
    }

    pub(crate) fn introspection(&self) -> bool {
        self.introspection
    }

    pub(crate) fn introspection_if(&self) -> Option<&LitStr> {
        self.introspection_if.as_ref()
    }

    pub(crate) fn schema(&self) -> Option<&LitStr> {
        self.schema.as_ref()
    }
}

impl Parse for Options {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let mut doc = DocString::new();
        let mut krate = parse_quote!(::aldrin);
        let mut client = true;
        let mut server = true;
        let mut introspection = false;
        let mut introspection_if = None;
        let mut schema = None;

        for attr in attrs {
            if attr.path().is_ident("doc") {
                doc.push(attr)?;
                continue;
            }

            if !attr.path().is_ident("aldrin") {
                return Err(Error::new_spanned(
                    attr,
                    "extected attributes `aldrin` or `doc`",
                ));
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    krate = meta.value()?.parse()?;
                    Ok(())
                } else if meta.path.is_ident("client") {
                    client = meta.value()?.parse::<LitBool>()?.value;
                    Ok(())
                } else if meta.path.is_ident("server") {
                    server = meta.value()?.parse::<LitBool>()?.value;
                    Ok(())
                } else if meta.path.is_ident("introspection") {
                    introspection = true;
                    Ok(())
                } else if meta.path.is_ident("introspection_if") {
                    introspection_if = meta.value()?.parse().map(Some)?;
                    introspection = true;
                    Ok(())
                } else if meta.path.is_ident("schema") {
                    schema = meta.value()?.parse().map(Some)?;
                    Ok(())
                } else {
                    Err(meta.error("unknown attribute"))
                }
            })?;
        }

        if !introspection || schema.is_some() {
            Ok(Self {
                doc,
                krate,
                client,
                server,
                introspection,
                introspection_if,
                schema,
            })
        } else {
            Err(input.error("the attribute `schema` is required to derive Introspectable"))
        }
    }
}
