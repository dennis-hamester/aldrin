use super::{ItemOptions, Options};
use crate::doc_string::DocString;
use quote::format_ident;
use syn::punctuated::Punctuated;
use syn::{
    DeriveInput, Error, Fields, Ident, LifetimeParam, Path, Result, Token, Type, Variant,
    Visibility, parse_quote,
};

pub(crate) struct EnumData<'a> {
    name: &'a Ident,
    options: Options,
    vis: &'a Visibility,
    variants: Vec<VariantData<'a>>,
    lifetimes: Vec<&'a LifetimeParam>,
}

impl<'a> EnumData<'a> {
    pub(crate) fn new(
        input: &'a DeriveInput,
        variants: &'a Punctuated<Variant, Token![,]>,
        krate: Path,
    ) -> Result<Self> {
        super::ensure_no_type_generics(&input.generics)?;

        let name = &input.ident;

        let mut variant_datas = Vec::with_capacity(variants.len());
        let mut fallback = false;
        let mut default_id = 0;
        for variant in variants {
            if fallback {
                return Err(Error::new_spanned(
                    variant,
                    "variants after the fallback are not allowed",
                ));
            }

            let variant_data =
                VariantData::new(variant, name, input.generics.lifetimes(), default_id)?;

            fallback = variant_data.is_fallback();
            default_id = variant_data.id() + 1;
            variant_datas.push(variant_data);
        }

        let options = Options::new(name, &input.attrs, krate, false)?;

        Ok(Self {
            name,
            vis: &input.vis,
            options,
            variants: variant_datas,
            lifetimes: input.generics.lifetimes().collect(),
        })
    }

    pub(crate) fn name(&self) -> &Ident {
        self.name
    }

    pub(crate) fn doc(&self) -> &DocString {
        self.options.doc()
    }

    pub(crate) fn krate(&self) -> &Path {
        self.options.krate()
    }

    pub(crate) fn vis(&self) -> &Visibility {
        self.vis
    }

    pub(crate) fn ref_type(&self) -> Option<&Ident> {
        self.options.ref_type()
    }

    pub(crate) fn variants(&self) -> &[VariantData<'_>] {
        &self.variants
    }

    pub(crate) fn ty_generics(&self) -> impl Iterator<Item = &Ident> {
        self.variants.iter().filter_map(VariantData::ty_generic)
    }

    pub(crate) fn lifetimes(&self) -> &[&LifetimeParam] {
        &self.lifetimes
    }
}

pub(crate) struct VariantData<'a> {
    name: &'a Ident,
    options: ItemOptions,
    ty: Option<&'a Type>,
    ty_generic: Option<Ident>,
    ty_tag: Option<Type>,
}

impl<'a> VariantData<'a> {
    fn new(
        variant: &'a Variant,
        enum_name: &Ident,
        lifetimes: impl Iterator<Item = &'a LifetimeParam>,
        default_id: u32,
    ) -> Result<Self> {
        let options = ItemOptions::new(&variant.attrs, default_id)?;

        if options.is_optional() {
            return Err(Error::new_spanned(
                variant,
                "enum variants cannot be optional",
            ));
        }

        let (ty, ty_generic) = match variant.fields {
            Fields::Unnamed(ref fields) if fields.unnamed.is_empty() => {
                if options.is_fallback() {
                    return Err(Error::new_spanned(
                        variant,
                        "the fallback variant must have exactly 1 element",
                    ));
                } else {
                    (None, None)
                }
            }

            Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => {
                let field = fields.unnamed.first().unwrap();
                let ty_generic = format_ident!("r#_{}", variant.ident);
                (Some(&field.ty), Some(ty_generic))
            }

            Fields::Unnamed(_) => {
                return Err(Error::new_spanned(
                    variant,
                    "tuple-like variants with more than 1 element are not supported by Aldrin",
                ));
            }

            Fields::Unit => {
                if options.is_fallback() {
                    return Err(Error::new_spanned(
                        variant,
                        "the fallback variant must have exactly 1 element",
                    ));
                } else {
                    (None, None)
                }
            }

            Fields::Named(_) => {
                return Err(Error::new_spanned(
                    variant,
                    "struct-like variants are not supported by Aldrin",
                ));
            }
        };

        let ty_tag = ty.map(|ty| {
            super::replace_self_ty(ty.clone(), parse_quote!(#enum_name<#(#lifetimes),*>))
        });

        Ok(Self {
            name: &variant.ident,
            options,
            ty,
            ty_generic,
            ty_tag,
        })
    }

    pub(crate) fn name(&self) -> &Ident {
        self.name
    }

    pub(crate) fn doc(&self) -> &DocString {
        self.options.doc()
    }

    pub(crate) fn id(&self) -> u32 {
        self.options.id()
    }

    pub(crate) fn is_fallback(&self) -> bool {
        self.options.is_fallback()
    }

    pub(crate) fn ty(&self) -> Option<&Type> {
        self.ty
    }

    pub(crate) fn ty_generic(&self) -> Option<&Ident> {
        self.ty_generic.as_ref()
    }

    pub(crate) fn ty_tag(&self) -> Option<&Type> {
        self.ty_tag.as_ref()
    }
}
