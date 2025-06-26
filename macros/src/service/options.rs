use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Attribute, Error, LitStr, Path, Result};

pub(super) struct Options {
    krate: Path,
    client: bool,
    server: bool,
    introspection: bool,
    introspection_if: Option<LitStr>,
    schema: Option<LitStr>,
}

impl Options {
    pub fn krate(&self) -> &Path {
        &self.krate
    }

    pub fn client(&self) -> bool {
        self.client
    }

    pub fn server(&self) -> bool {
        self.server
    }

    pub fn introspection(&self) -> bool {
        self.introspection
    }

    pub fn introspection_if(&self) -> Option<&LitStr> {
        self.introspection_if.as_ref()
    }

    pub fn schema(&self) -> Option<&LitStr> {
        self.schema.as_ref()
    }
}

impl Parse for Options {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let mut krate = parse_quote!(::aldrin);
        let mut client = true;
        let mut server = true;
        let mut introspection = false;
        let mut introspection_if = None;
        let mut schema = None;

        for attr in attrs {
            if !attr.path().is_ident("aldrin") {
                return Err(Error::new_spanned(attr, "extected attribute `aldrin`"));
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    krate = meta.value()?.parse()?;
                    Ok(())
                } else if meta.path.is_ident("no_client") {
                    client = false;
                    Ok(())
                } else if meta.path.is_ident("no_server") {
                    server = false;
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
