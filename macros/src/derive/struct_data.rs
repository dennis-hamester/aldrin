use super::{ItemOptions, Options};
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens};
use std::borrow::Cow;
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::{
    parse_quote, DeriveInput, Error, Field, Ident, Index, LifetimeParam, Path, Result, Token, Type,
    Visibility,
};

pub(crate) struct StructData<'a> {
    name: &'a Ident,
    options: Options,
    named: bool,
    vis: &'a Visibility,
    fields: Vec<FieldData<'a>>,
    lifetimes: Vec<&'a LifetimeParam>,
}

impl<'a> StructData<'a> {
    pub fn new(
        input: &'a DeriveInput,
        named: bool,
        fields: &'a Punctuated<Field, Token![,]>,
        krate: Path,
    ) -> Result<Self> {
        super::ensure_no_type_generics(&input.generics)?;

        let name = &input.ident;
        let options = Options::new(name, &input.attrs, krate, true)?;
        let mut field_datas = Vec::with_capacity(fields.len());
        let mut fallback = false;
        let mut default_id = 0;

        for (index, field) in fields.into_iter().enumerate() {
            if fallback {
                return Err(Error::new_spanned(
                    field,
                    "fields after the fallback are not allowed",
                ));
            }

            let field_data =
                FieldData::new(field, index, name, input.generics.lifetimes(), default_id)?;

            if options.newtype() {
                if field_data.is_fallback() {
                    return Err(Error::new_spanned(
                        field,
                        "a fallback field is not supported for newtype structs",
                    ));
                }

                if field_data.is_optional() {
                    return Err(Error::new_spanned(
                        field,
                        "an optional field is not supported for newtype structs",
                    ));
                }
            }

            if field_data.is_fallback() {
                if field_data.is_optional() {
                    return Err(Error::new_spanned(
                        field,
                        "fields cannot be marked both optional and fallback",
                    ));
                }

                fallback = true;
            }

            default_id = field_data.id() + 1;
            field_datas.push(field_data);
        }

        if options.newtype() && (field_datas.len() != 1) {
            return Err(Error::new_spanned(
                input,
                "newtype structs must have exactly 1 field",
            ));
        }

        Ok(Self {
            name,
            named,
            vis: &input.vis,
            options,
            fields: field_datas,
            lifetimes: input.generics.lifetimes().collect(),
        })
    }

    pub fn name(&self) -> &Ident {
        self.name
    }

    pub fn krate(&self) -> &Path {
        self.options.krate()
    }

    pub fn is_named(&self) -> bool {
        self.named
    }

    pub fn vis(&self) -> &Visibility {
        self.vis
    }

    pub fn ref_type(&self) -> Option<&Ident> {
        self.options.ref_type()
    }

    pub fn is_newtype(&self) -> bool {
        self.options.newtype()
    }

    pub fn fields(&self) -> &[FieldData<'_>] {
        &self.fields
    }

    pub fn ty_generics(&self) -> impl Iterator<Item = &Ident> {
        self.fields.iter().map(FieldData::ty_generic)
    }

    pub fn lifetimes(&self) -> &[&LifetimeParam] {
        &self.lifetimes
    }

    pub fn fallback(&self) -> Option<&FieldData<'_>> {
        match self.fields.last() {
            Some(field) if field.is_fallback() => Some(field),
            _ => None,
        }
    }
}

pub(crate) struct FieldData<'a> {
    name: FieldName<'a>,
    options: ItemOptions,
    ty: &'a Type,
    ty_generic: Ident,
    ty_tag: Type,
    var: Cow<'a, Ident>,
}

impl<'a> FieldData<'a> {
    fn new(
        field: &'a Field,
        index: usize,
        struct_name: &Ident,
        lifetimes: impl Iterator<Item = &'a LifetimeParam>,
        default_id: u32,
    ) -> Result<Self> {
        let options = ItemOptions::new(&field.attrs, default_id)?;
        let name = FieldName::new(field.ident.as_ref(), index);

        let ty_generic = match name {
            FieldName::Ident(name) => format_ident!(
                "r#_{}",
                name.unraw().to_string().to_upper_camel_case(),
                span = name.span()
            ),

            FieldName::Index(ref index) => format_ident!("r#_Field{}", index.index),
        };

        let ty_tag = super::replace_self_ty(
            field.ty.clone(),
            parse_quote!(#struct_name<#(#lifetimes),*>),
        );

        let var = match name {
            FieldName::Ident(name) => Cow::Borrowed(name),
            FieldName::Index(ref index) => Cow::Owned(format_ident!("r#_field{}", index.index)),
        };

        Ok(Self {
            name,
            options,
            ty: &field.ty,
            ty_generic,
            ty_tag,
            var,
        })
    }

    pub fn is_named(&self) -> bool {
        matches!(self.name, FieldName::Ident(_))
    }

    pub fn name(&self) -> &FieldName<'_> {
        &self.name
    }

    pub fn id(&self) -> u32 {
        self.options.id()
    }

    pub fn is_fallback(&self) -> bool {
        self.options.is_fallback()
    }

    pub fn is_optional(&self) -> bool {
        self.options.is_optional()
    }

    pub fn ty(&self) -> &Type {
        self.ty
    }

    pub fn ty_generic(&self) -> &Ident {
        &self.ty_generic
    }

    pub fn ty_tag(&self) -> &Type {
        &self.ty_tag
    }

    pub fn var(&self) -> &Ident {
        &self.var
    }
}

pub(crate) enum FieldName<'a> {
    Ident(&'a Ident),
    Index(Index),
}

impl<'a> FieldName<'a> {
    fn new(ident: Option<&'a Ident>, index: usize) -> Self {
        match ident {
            Some(ident) => Self::Ident(ident),
            None => Self::Index(index.into()),
        }
    }
}

impl ToTokens for FieldName<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::Index(index) => index.to_tokens(tokens),
        }
    }
}
