use super::{ItemOptions, Options};
use quote::format_ident;
use syn::punctuated::Punctuated;
use syn::{
    parse_quote, DeriveInput, Error, Fields, Ident, LifetimeParam, Path, Result, Token, Type,
    Variant, Visibility,
};

pub(crate) struct EnumData<'a> {
    name: &'a Ident,
    options: Options,
    vis: &'a Visibility,
    ref_type: Result<Ident>,
    variants: Vec<VariantData<'a>>,
    lifetimes: Vec<&'a LifetimeParam>,
}

impl<'a> EnumData<'a> {
    pub fn new(
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

        let options = Options::new(&input.attrs, krate)?;

        Ok(Self {
            name,
            vis: &input.vis,
            ref_type: options.ref_type(&input.ident),
            options,
            variants: variant_datas,
            lifetimes: input.generics.lifetimes().collect(),
        })
    }

    pub fn name(&self) -> &Ident {
        self.name
    }

    pub fn krate(&self) -> &Path {
        self.options.krate()
    }

    pub fn vis(&self) -> &Visibility {
        self.vis
    }

    pub fn ref_type(&self) -> Result<&Ident> {
        match self.ref_type {
            Ok(ref ref_type) => Ok(ref_type),
            Err(ref e) => Err(e.clone()),
        }
    }

    pub fn variants(&self) -> &[VariantData] {
        &self.variants
    }

    pub fn ty_generics(&self) -> impl Iterator<Item = &Ident> {
        self.variants.iter().filter_map(VariantData::ty_generic)
    }

    pub fn lifetimes(&self) -> &[&LifetimeParam] {
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
                ))
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
                ))
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

    pub fn name(&self) -> &Ident {
        self.name
    }

    pub fn id(&self) -> u32 {
        self.options.id()
    }

    pub fn is_fallback(&self) -> bool {
        self.options.is_fallback()
    }

    pub fn ty(&self) -> Option<&Type> {
        self.ty
    }

    pub fn ty_generic(&self) -> Option<&Ident> {
        self.ty_generic.as_ref()
    }

    pub fn ty_tag(&self) -> Option<&Type> {
        self.ty_tag.as_ref()
    }
}
