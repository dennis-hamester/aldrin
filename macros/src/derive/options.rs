use crate::doc_string::DocString;
use aldrin_codegen::rust::names;
use syn::ext::IdentExt;
use syn::{Attribute, Ident, LitInt, LitStr, Path, Result};

pub(crate) struct Options {
    doc: DocString,
    krate: Path,
    ref_type: Option<Ident>,
    schema: Option<LitStr>,
    newtype: bool,
}

impl Options {
    pub(crate) fn new(
        name: &Ident,
        attrs: &[Attribute],
        mut krate: Path,
        is_struct: bool,
    ) -> Result<Self> {
        let mut doc = DocString::new();
        let mut ref_type = None;
        let mut schema = None;
        let mut newtype = false;

        for attr in attrs {
            if attr.path().is_ident("doc") {
                let _ = doc.push(attr.clone());
                continue;
            }

            if !attr.path().is_ident("aldrin") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    krate = meta.value()?.parse()?;
                    Ok(())
                } else if meta.path.is_ident("ref_type") {
                    if meta.input.is_empty() {
                        ref_type = Some(Ident::new_raw(
                            &names::default_ref_type(&name.unraw().to_string()),
                            name.span(),
                        ));
                    } else {
                        ref_type = meta.value()?.parse().map(Some)?;
                    }
                    Ok(())
                } else if meta.path.is_ident("schema") {
                    schema = meta.value()?.parse().map(Some)?;
                    Ok(())
                } else if meta.path.is_ident("newtype") {
                    if is_struct {
                        newtype = true;
                        Ok(())
                    } else {
                        Err(meta.error("`newtype` is supported only for structs"))
                    }
                } else {
                    Err(meta.error("unknown attribute"))
                }
            })?;
        }

        Ok(Self {
            doc,
            krate,
            ref_type,
            schema,
            newtype,
        })
    }

    pub(crate) fn doc(&self) -> &DocString {
        &self.doc
    }

    pub(crate) fn krate(&self) -> &Path {
        &self.krate
    }

    pub(crate) fn ref_type(&self) -> Option<&Ident> {
        self.ref_type.as_ref()
    }

    pub(crate) fn schema(&self) -> Option<&LitStr> {
        self.schema.as_ref()
    }

    pub(crate) fn newtype(&self) -> bool {
        self.newtype
    }
}

pub(crate) struct ItemOptions {
    doc: DocString,
    id: u32,
    optional: bool,
    fallback: bool,
}

impl ItemOptions {
    pub(crate) fn new(attrs: &[Attribute], default_id: u32) -> Result<Self> {
        let mut doc = DocString::new();
        let mut id = default_id;
        let mut optional = false;
        let mut fallback = false;

        for attr in attrs {
            if attr.path().is_ident("doc") {
                let _ = doc.push(attr.clone());
                continue;
            }

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
            doc,
            id,
            optional,
            fallback,
        })
    }

    pub(crate) fn doc(&self) -> &DocString {
        &self.doc
    }

    pub(crate) fn id(&self) -> u32 {
        self.id
    }

    pub(crate) fn is_optional(&self) -> bool {
        self.optional
    }

    pub(crate) fn is_fallback(&self) -> bool {
        self.fallback
    }
}
