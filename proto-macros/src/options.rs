use syn::punctuated::Punctuated;
use syn::{Attribute, Error, LitInt, LitStr, Path, Result, Token, WherePredicate};

pub struct Options {
    krate: Path,
    ser_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    de_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    deny_unknown_fields: bool,
}

impl Options {
    pub fn new(attrs: &[Attribute]) -> Result<Self> {
        let mut krate = None;
        let mut ser_bounds = None;
        let mut de_bounds = None;
        let mut deny_unknown_fields = false;

        for attr in attrs {
            if !attr.path().is_ident("aldrin") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    let value: LitStr = meta.value()?.parse()?;
                    krate = value
                        .parse()
                        .map(Some)
                        .map_err(|e| Error::new_spanned(value, e))?;
                    Ok(())
                } else if meta.path.is_ident("bounds") {
                    let value: LitStr = meta.value()?.parse()?;
                    ser_bounds = parse_lit_str_into_where_predicates(&value).map(Some)?;
                    de_bounds = ser_bounds.clone();
                    Ok(())
                } else if meta.path.is_ident("ser_bounds") {
                    let value: LitStr = meta.value()?.parse()?;
                    ser_bounds = parse_lit_str_into_where_predicates(&value).map(Some)?;
                    Ok(())
                } else if meta.path.is_ident("de_bounds") {
                    let value: LitStr = meta.value()?.parse()?;
                    de_bounds = parse_lit_str_into_where_predicates(&value).map(Some)?;
                    Ok(())
                } else if meta.path.is_ident("deny_unknown_fields") {
                    if meta.input.is_empty() {
                        deny_unknown_fields = true;
                        Ok(())
                    } else {
                        Err(meta.input.error("unexpected extra tokens"))
                    }
                } else {
                    Err(meta.error("unknown attribute"))
                }
            })?;
        }

        Ok(Self {
            krate: krate.unwrap_or_else(|| syn::parse_quote!(aldrin_proto)),
            ser_bounds,
            de_bounds,
            deny_unknown_fields,
        })
    }

    pub fn krate(&self) -> &Path {
        &self.krate
    }

    pub fn ser_bounds(&self) -> Option<&Punctuated<WherePredicate, Token![,]>> {
        self.ser_bounds.as_ref()
    }

    pub fn de_bounds(&self) -> Option<&Punctuated<WherePredicate, Token![,]>> {
        self.de_bounds.as_ref()
    }

    pub fn deny_unknown_fields(&self) -> bool {
        self.deny_unknown_fields
    }
}

pub struct ItemOptions {
    id: u32,
}

impl ItemOptions {
    pub fn new(attrs: &[Attribute], default_id: u32) -> Result<Self> {
        let mut id = default_id;

        for attr in attrs {
            if !attr.path().is_ident("aldrin") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("id") {
                    let value: LitInt = meta.value()?.parse()?;
                    id = value.base10_parse()?;
                    Ok(())
                } else {
                    Err(meta.error("unknown attribute"))
                }
            })?;
        }

        Ok(Self { id })
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

fn parse_lit_str_into_where_predicates(
    lit_str: &LitStr,
) -> Result<Punctuated<WherePredicate, Token![,]>> {
    let parser = Punctuated::<WherePredicate, Token![,]>::parse_terminated;
    lit_str
        .parse_with(parser)
        .map_err(|e| Error::new_spanned(lit_str, e))
}
