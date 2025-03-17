use syn::ext::IdentExt;
use syn::{Attribute, Error, Ident, LitInt, LitStr, Path, Result};

pub(crate) struct Options {
    krate: Path,
    ref_type: RefType,
    schema: Option<LitStr>,
}

impl Options {
    pub fn new(attrs: &[Attribute], mut krate: Path) -> Result<Self> {
        let mut ref_type = RefType::Default;
        let mut schema = None;

        for attr in attrs {
            if !attr.path().is_ident("aldrin") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    krate = meta.value()?.parse::<LitStr>()?.parse()?;
                    Ok(())
                } else if meta.path.is_ident("ref_type") {
                    let value: LitStr = meta.value()?.parse()?;

                    match value.parse()? {
                        Some(ident) => ref_type = RefType::Custom(ident),

                        None => {
                            ref_type = RefType::Disabled(meta.error("ref type has been disabled"))
                        }
                    }

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
            ref_type,
            schema,
        })
    }

    pub fn krate(&self) -> &Path {
        &self.krate
    }

    pub fn ref_type(&self, base: &Ident) -> Result<Ident> {
        match self.ref_type {
            RefType::Default => Ok(Ident::new_raw(&format!("{}Ref", base.unraw()), base.span())),
            RefType::Disabled(ref err) => Err(err.clone()),
            RefType::Custom(ref ident) => Ok(ident.clone()),
        }
    }

    pub fn schema(&self) -> Option<&LitStr> {
        self.schema.as_ref()
    }
}

enum RefType {
    Default,
    Disabled(Error),
    Custom(Ident),
}

pub(crate) struct ItemOptions {
    id: u32,
    optional: bool,
    fallback: bool,
}

impl ItemOptions {
    pub fn new(attrs: &[Attribute], default_id: u32) -> Result<Self> {
        let mut id = default_id;
        let mut optional = false;
        let mut fallback = false;

        for attr in attrs {
            if !attr.path().is_ident("aldrin") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("id") {
                    id = meta.value()?.parse::<LitInt>()?.base10_parse()?;
                    Ok(())
                } else if meta.path.is_ident("optional") {
                    optional = true;
                    Ok(())
                } else if meta.path.is_ident("fallback") {
                    fallback = true;
                    Ok(())
                } else {
                    Err(meta.error("unknown attribute"))
                }
            })?;
        }

        Ok(Self {
            id,
            optional,
            fallback,
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn is_optional(&self) -> bool {
        self.optional
    }

    pub fn is_fallback(&self) -> bool {
        self.fallback
    }
}
