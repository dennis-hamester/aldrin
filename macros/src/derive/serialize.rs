use super::{EnumData, FieldData, StructData, VariantData};
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    parse_quote, Data, DeriveInput, Error, Field, Fields, GenericParam, Ident, Path, Result, Token,
    Variant,
};

pub fn gen_serialize_from_core(input: DeriveInput) -> Result<TokenStream> {
    gen_serialize(input, parse_quote!(::aldrin_core))
}

pub fn gen_serialize_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    gen_serialize(input, parse_quote!(::aldrin::core))
}

fn gen_serialize(input: DeriveInput, krate: Path) -> Result<TokenStream> {
    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => gen_struct(&input, true, &fields.named, krate),
            Fields::Unnamed(ref fields) => gen_struct(&input, false, &fields.unnamed, krate),
            Fields::Unit => gen_struct(&input, false, &Punctuated::new(), krate),
        },

        Data::Enum(ref data) => gen_enum(&input, &data.variants, krate),

        Data::Union(_) => Err(Error::new_spanned(
            input.ident,
            "unions are not supported by Aldrin",
        )),
    }
}

fn gen_struct(
    input: &DeriveInput,
    named: bool,
    fields: &Punctuated<Field, Token![,]>,
    krate: Path,
) -> Result<TokenStream> {
    StructData::new(input, named, fields, krate)?.gen_serialize()
}

impl StructData<'_> {
    fn gen_serialize(&self) -> Result<TokenStream> {
        let for_self = self.gen_serialize_for_self()?;
        let for_ref = self.gen_serialize_for_ref()?;

        let for_ref_type = if self.ref_type().is_ok() {
            self.gen_serialize_for_ref_type().map(Some)?
        } else {
            None
        };

        Ok(quote! {
            #for_self
            #for_ref
            #for_ref_type
        })
    }

    fn gen_serialize_for_self(&self) -> Result<TokenStream> {
        let name = self.name();
        let lifetimes = self.lifetimes();
        let ty = quote! { #name<#(#lifetimes),*> };
        let krate = self.krate();
        let lifetimes = self.lifetimes();

        let serializer = match self.fallback() {
            Some(fallback) => {
                let name = fallback.name();

                quote! { serializer.serialize_struct2_with_unknown_fields(self.#name)? }
            }

            None => quote! { serializer.serialize_struct2()? },
        };

        let fields = self
            .fields()
            .iter()
            .filter_map(|field| field.gen_serialize_for_self(krate));

        Ok(quote! {
            #[automatically_derived]
            impl<#(#lifetimes),*> #krate::Serialize<Self> for #ty {
                fn serialize(
                    self,
                    serializer: #krate::Serializer,
                ) -> ::std::result::Result<(), #krate::SerializeError> {
                    let mut serializer = #serializer;
                    #(#fields)*
                    serializer.finish()
                }
            }
        })
    }

    fn gen_serialize_for_ref(&self) -> Result<TokenStream> {
        let name = self.name();
        let lifetimes = self.lifetimes();
        let ty = quote! { #name<#(#lifetimes),*> };
        let krate = self.krate();
        let lifetimes = self.lifetimes();

        let serializer = match self.fallback() {
            Some(fallback) => {
                let name = fallback.name();

                quote! { serializer.serialize_struct2_with_unknown_fields(&self.#name)? }
            }

            None => quote! { serializer.serialize_struct2()? },
        };

        let fields = self
            .fields()
            .iter()
            .filter_map(|field| field.gen_serialize_for_ref(krate));

        Ok(quote! {
            #[automatically_derived]
            impl<'_self, #(#lifetimes),*> #krate::Serialize<#ty> for &'_self #ty {
                fn serialize(
                    self,
                    serializer: #krate::Serializer,
                ) -> ::std::result::Result<(), #krate::SerializeError> {
                    let mut serializer = #serializer;
                    #(#fields)*
                    serializer.finish()
                }
            }
        })
    }

    fn gen_serialize_for_ref_type(&self) -> Result<TokenStream> {
        let name = self.name();
        let ref_type = self.ref_type()?;
        let krate = self.krate();
        let lifetimes = self.lifetimes();
        let ty_generics = self.ty_generics();

        let where_bounds = self
            .fields()
            .iter()
            .map(|field| field.serialize_where_bound_for_ref_type(krate));

        let serializer = match self.fallback() {
            Some(fallback) => {
                let name = fallback.name();

                quote! { serializer.serialize_struct2_with_unknown_fields(self.#name)? }
            }

            None => quote! { serializer.serialize_struct2()? },
        };

        let fields = self
            .fields()
            .iter()
            .filter_map(|field| field.gen_serialize_for_ref_type(krate));

        let impl_generics = self
            .lifetimes()
            .iter()
            .map(|lt| GenericParam::Lifetime((*lt).clone()))
            .chain(
                self.ty_generics()
                    .map(|ty| GenericParam::Type(ty.clone().into())),
            );

        Ok(quote! {
            #[automatically_derived]
            impl<#(#impl_generics),*> #krate::Serialize<#name<#(#lifetimes),*>> for #ref_type<#(#ty_generics),*>
            where
                #(#where_bounds),*
            {
                fn serialize(
                    self,
                    serializer: #krate::Serializer,
                ) -> ::std::result::Result<(), #krate::SerializeError> {
                    let mut serializer = #serializer;
                    #(#fields)*
                    serializer.finish()
                }
            }
        })
    }
}

impl FieldData<'_> {
    fn gen_serialize_for_self(&self, krate: &Path) -> Option<TokenStream> {
        if self.is_fallback() {
            return None;
        }

        let ty = self.ty();
        let id = self.id();
        let name = self.name();

        let serialize = if self.is_optional() {
            quote! { serialize_if_some }
        } else {
            quote! { serialize }
        };

        Some(quote! {
            serializer.#serialize::<#krate::tags::As<#ty>, _>(#id, self.#name)?;
        })
    }

    fn gen_serialize_for_ref(&self, krate: &Path) -> Option<TokenStream> {
        if self.is_fallback() {
            return None;
        }

        let ty_tag = self.ty_tag();
        let id = self.id();
        let name = self.name();

        let serialize = if self.is_optional() {
            quote! { serialize_if_some }
        } else {
            quote! { serialize }
        };

        Some(quote! {
            serializer.#serialize::<#krate::tags::As<#ty_tag>, _>(#id, &self.#name)?;
        })
    }

    fn gen_serialize_for_ref_type(&self, krate: &Path) -> Option<TokenStream> {
        if self.is_fallback() {
            return None;
        }

        let ty_tag = self.ty_tag();
        let id = self.id();
        let name = self.name();

        let serialize = if self.is_optional() {
            quote! { serialize_if_some }
        } else {
            quote! { serialize }
        };

        Some(quote! {
            serializer.#serialize::<#krate::tags::As<#ty_tag>, _>(#id, self.#name)?;
        })
    }

    fn serialize_where_bound_for_ref_type(&self, krate: &Path) -> TokenStream {
        let ty_generic = self.ty_generic();

        if self.is_fallback() {
            quote! { #ty_generic: #krate::AsUnknownFields }
        } else {
            let ty_tag = self.ty_tag();

            quote! { #ty_generic: #krate::Serialize<#krate::tags::As<#ty_tag>> }
        }
    }
}

fn gen_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Token![,]>,
    krate: Path,
) -> Result<TokenStream> {
    EnumData::new(input, variants, krate)?.gen_serialize()
}

impl EnumData<'_> {
    fn gen_serialize(&self) -> Result<TokenStream> {
        let for_self = self.gen_serialize_for_self()?;
        let for_ref = self.gen_serialize_for_ref()?;

        let for_ref_type = if self.ref_type().is_ok() {
            self.gen_serialize_for_ref_type().map(Some)?
        } else {
            None
        };

        Ok(quote! {
            #for_self
            #for_ref
            #for_ref_type
        })
    }

    fn gen_serialize_for_self(&self) -> Result<TokenStream> {
        let name = self.name();
        let lifetimes = self.lifetimes();
        let ty = quote! { #name<#(#lifetimes),*> };
        let krate = self.krate();
        let lifetimes = self.lifetimes();

        let variants = self
            .variants()
            .iter()
            .map(|var| var.gen_serialize_for_self(krate));

        Ok(quote! {
            #[automatically_derived]
            impl<#(#lifetimes),*> #krate::Serialize<Self> for #ty {
                fn serialize(
                    self,
                    serializer: #krate::Serializer,
                ) -> ::std::result::Result<(), #krate::SerializeError> {
                    match self {
                        #(#variants)*
                    }
                }
            }
        })
    }

    fn gen_serialize_for_ref(&self) -> Result<TokenStream> {
        let name = self.name();
        let lifetimes = self.lifetimes();
        let ty = quote! { #name<#(#lifetimes),*> };
        let krate = self.krate();
        let lifetimes = self.lifetimes();

        let variants = self
            .variants()
            .iter()
            .map(|var| var.gen_serialize_for_ref(name, krate));

        Ok(quote! {
            #[automatically_derived]
            impl<'_self, #(#lifetimes),*> #krate::Serialize<#ty> for &'_self #ty {
                fn serialize(
                    self,
                    serializer: #krate::Serializer,
                ) -> ::std::result::Result<(), #krate::SerializeError> {
                    match *self {
                        #(#variants)*
                    }
                }
            }
        })
    }

    fn gen_serialize_for_ref_type(&self) -> Result<TokenStream> {
        let name = self.name();
        let ref_type = self.ref_type()?;
        let krate = self.krate();
        let lifetimes = self.lifetimes();
        let ty_generics = self.ty_generics();

        let where_bounds = self
            .variants()
            .iter()
            .filter_map(|var| var.serialize_where_bound_for_ref_type(krate));

        let variants = self
            .variants()
            .iter()
            .map(|var| var.gen_serialize_for_ref_type(krate));

        let impl_generics = self
            .lifetimes()
            .iter()
            .map(|lt| GenericParam::Lifetime((*lt).clone()))
            .chain(
                self.ty_generics()
                    .map(|ty| GenericParam::Type(ty.clone().into())),
            );

        Ok(quote! {
            #[automatically_derived]
            impl<#(#impl_generics),*> #krate::Serialize<#name<#(#lifetimes),*>> for #ref_type<#(#ty_generics),*>
            where
                #(#where_bounds),*
            {
                fn serialize(
                    self,
                    serializer: #krate::Serializer,
                ) -> ::std::result::Result<(), #krate::SerializeError> {
                    match self {
                        #(#variants)*
                    }
                }
            }
        })
    }
}

impl VariantData<'_> {
    fn gen_serialize_for_self(&self, krate: &Path) -> TokenStream {
        let name = self.name();
        let id = self.id();

        if self.is_fallback() {
            quote! {
                Self::#name(_value) => {
                    serializer.serialize_unknown_variant(_value)
                }
            }
        } else if let Some(ty) = self.ty() {
            quote! {
                Self::#name(_value) => {
                    serializer.serialize_enum::<#krate::tags::As<#ty>, _>(#id, _value)
                }
            }
        } else {
            quote! {
                Self::#name {} => serializer.serialize_unit_enum(#id),
            }
        }
    }

    fn gen_serialize_for_ref(&self, enum_name: &Ident, krate: &Path) -> TokenStream {
        let name = self.name();
        let id = self.id();

        if self.is_fallback() {
            quote! {
                #enum_name::#name(ref _value) => {
                    serializer.serialize_unknown_variant(_value)
                }
            }
        } else if let Some(ty_tag) = self.ty_tag() {
            quote! {
                #enum_name::#name(ref _value) => {
                    serializer.serialize_enum::<#krate::tags::As<#ty_tag>, _>(#id, _value)
                }
            }
        } else {
            quote! {
                #enum_name::#name {} => serializer.serialize_unit_enum(#id),
            }
        }
    }

    fn gen_serialize_for_ref_type(&self, krate: &Path) -> TokenStream {
        let name = self.name();
        let id = self.id();

        if self.is_fallback() {
            quote! {
                Self::#name(_value) => {
                    serializer.serialize_unknown_variant(_value)
                }
            }
        } else if let Some(ty_tag) = self.ty_tag() {
            quote! {
                Self::#name(_value) => {
                    serializer.serialize_enum::<#krate::tags::As<#ty_tag>, _>(#id, _value)
                }
            }
        } else {
            quote! {
                Self::#name => serializer.serialize_unit_enum(#id),
            }
        }
    }

    fn serialize_where_bound_for_ref_type(&self, krate: &Path) -> Option<TokenStream> {
        let ty_generic = self.ty_generic()?;

        if self.is_fallback() {
            Some(quote! {
                #ty_generic: #krate::AsUnknownVariant
            })
        } else {
            let ty_tag = self.ty_tag()?;

            Some(quote! {
                #ty_generic: #krate::Serialize<#krate::tags::As<#ty_tag>>
            })
        }
    }
}
