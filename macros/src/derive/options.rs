use syn::punctuated::Punctuated;
use syn::{Attribute, Error, LitInt, LitStr, Path, Result, Token, WherePredicate};

pub struct Options {
    krate: Path,
    ser_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    de_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    intro_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    ser_key_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    de_key_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    key_ty_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    schema: Option<LitStr>,
}

impl Options {
    pub fn new(attrs: &[Attribute], mut krate: Path) -> Result<Self> {
        let mut ser_bounds = None;
        let mut de_bounds = None;
        let mut intro_bounds = None;
        let mut ser_key_bounds = None;
        let mut de_key_bounds = None;
        let mut key_ty_bounds = None;
        let mut schema = None;

        for attr in attrs {
            if !attr.path().is_ident("aldrin") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    let value: LitStr = meta.value()?.parse()?;
                    krate = value.parse().map_err(|e| Error::new_spanned(value, e))?;
                    Ok(())
                } else if meta.path.is_ident("bounds") {
                    let value: LitStr = meta.value()?.parse()?;
                    ser_bounds = parse_lit_str_into_where_predicates(&value).map(Some)?;
                    de_bounds = ser_bounds.clone();
                    intro_bounds = ser_bounds.clone();
                    ser_key_bounds = ser_bounds.clone();
                    de_key_bounds = ser_bounds.clone();
                    key_ty_bounds = ser_bounds.clone();
                    Ok(())
                } else if meta.path.is_ident("ser_bounds") {
                    let value: LitStr = meta.value()?.parse()?;
                    ser_bounds = parse_lit_str_into_where_predicates(&value).map(Some)?;
                    Ok(())
                } else if meta.path.is_ident("de_bounds") {
                    let value: LitStr = meta.value()?.parse()?;
                    de_bounds = parse_lit_str_into_where_predicates(&value).map(Some)?;
                    Ok(())
                } else if meta.path.is_ident("intro_bounds") {
                    let value: LitStr = meta.value()?.parse()?;
                    intro_bounds = parse_lit_str_into_where_predicates(&value).map(Some)?;
                    Ok(())
                } else if meta.path.is_ident("ser_key_bounds") {
                    let value: LitStr = meta.value()?.parse()?;
                    ser_key_bounds = parse_lit_str_into_where_predicates(&value).map(Some)?;
                    Ok(())
                } else if meta.path.is_ident("de_key_bounds") {
                    let value: LitStr = meta.value()?.parse()?;
                    de_key_bounds = parse_lit_str_into_where_predicates(&value).map(Some)?;
                    Ok(())
                } else if meta.path.is_ident("key_ty_bounds") {
                    let value: LitStr = meta.value()?.parse()?;
                    key_ty_bounds = parse_lit_str_into_where_predicates(&value).map(Some)?;
                    Ok(())
                } else if meta.path.is_ident("schema") {
                    schema = meta.value()?.parse().map(Some)?;
                    Ok(())
                } else {
                    Err(meta.error("unknown attribute"))
                }
            })?;
        }

        Ok(Self {
            krate,
            ser_bounds,
            de_bounds,
            intro_bounds,
            ser_key_bounds,
            de_key_bounds,
            key_ty_bounds,
            schema,
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

    pub fn intro_bounds(&self) -> Option<&Punctuated<WherePredicate, Token![,]>> {
        self.intro_bounds.as_ref()
    }

    pub fn ser_key_bounds(&self) -> Option<&Punctuated<WherePredicate, Token![,]>> {
        self.ser_key_bounds.as_ref()
    }

    pub fn de_key_bounds(&self) -> Option<&Punctuated<WherePredicate, Token![,]>> {
        self.de_key_bounds.as_ref()
    }

    pub fn key_ty_bounds(&self) -> Option<&Punctuated<WherePredicate, Token![,]>> {
        self.key_ty_bounds.as_ref()
    }

    pub fn schema(&self) -> Option<&LitStr> {
        self.schema.as_ref()
    }
}

pub struct ItemOptions {
    id: u32,
    optional: bool,
}

impl ItemOptions {
    pub fn new(attrs: &[Attribute], default_id: u32) -> Result<Self> {
        let mut id = default_id;
        let mut optional = false;

        for attr in attrs {
            if !attr.path().is_ident("aldrin") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("id") {
                    let value: LitInt = meta.value()?.parse()?;
                    id = value.base10_parse()?;
                    Ok(())
                } else if meta.path.is_ident("optional") {
                    optional = true;
                    Ok(())
                } else {
                    Err(meta.error("unknown attribute"))
                }
            })?;
        }

        Ok(Self { id, optional })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn is_optional(&self) -> bool {
        self.optional
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
