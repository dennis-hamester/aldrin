use std::fmt::Display;
use std::str::FromStr;
use syn::punctuated::Punctuated;
use syn::{Attribute, Error, Lit, LitInt, LitStr, Meta, NestedMeta, Path, Result, Token};

pub struct Options {
    krate: Path,
}

impl Options {
    pub fn new(attrs: &[Attribute]) -> Result<Self> {
        let mut krate = None;

        parse_nested_metas_with(attrs, |meta| match meta {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("crate") => {
                krate = Some(parse_lit_into_path(&nv.lit)?);
                Ok(())
            }

            _ => Err(Error::new_spanned(meta, "unknown attribute")),
        })?;

        Ok(Self {
            krate: krate.unwrap_or_else(|| syn::parse_quote!(aldrin_proto)),
        })
    }

    pub fn krate(&self) -> &Path {
        &self.krate
    }
}

pub struct ItemOptions {
    id: u32,
}

impl ItemOptions {
    pub fn new(attrs: &[Attribute], default_id: u32) -> Result<Self> {
        let mut id = default_id;

        parse_nested_metas_with(attrs, |meta| match meta {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("id") => {
                id = parse_lit_int(&nv.lit)?;
                Ok(())
            }

            _ => Err(Error::new_spanned(meta, "unknown attribute")),
        })?;

        Ok(Self { id })
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

fn parse_nested_metas_with<F>(attrs: &[Attribute], mut f: F) -> Result<()>
where
    F: FnMut(NestedMeta) -> Result<()>,
{
    for attr in attrs {
        if !attr.path.is_ident("aldrin") {
            continue;
        }

        let metas = get_nested_attr_metas(attr)?;

        for meta in metas {
            f(meta)?;
        }
    }

    Ok(())
}

fn get_nested_attr_metas(attr: &Attribute) -> Result<Punctuated<NestedMeta, Token![,]>> {
    let meta = attr.parse_meta()?;
    let Meta::List(metas) = meta else {
        return Err(Error::new_spanned(attr, "expected #[aldrin(...)]"));
    };

    Ok(metas.nested)
}

fn get_lit_str(lit: &Lit) -> Result<&LitStr> {
    if let Lit::Str(lit) = lit {
        Ok(lit)
    } else {
        Err(Error::new_spanned(lit, "expected a string literal"))
    }
}

fn get_lit_int(lit: &Lit) -> Result<&LitInt> {
    if let Lit::Int(lit) = lit {
        Ok(lit)
    } else {
        Err(Error::new_spanned(lit, "expected an integer literal"))
    }
}

fn parse_lit_into_path(lit: &Lit) -> Result<Path> {
    let string = get_lit_str(lit)?;
    string.parse().map_err(|e| Error::new_spanned(string, e))
}

fn parse_lit_int<T>(lit: &Lit) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    let int = get_lit_int(lit)?;
    int.base10_parse()
}
