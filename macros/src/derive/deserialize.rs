use super::{EnumData, FieldData, StructData, VariantData};
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{parse_quote, Data, DeriveInput, Error, Field, Fields, Path, Result, Token, Variant};

pub fn gen_deserialize_from_core(input: DeriveInput) -> Result<TokenStream> {
    gen_deserialize(input, parse_quote!(::aldrin_core))
}

pub fn gen_deserialize_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    gen_deserialize(input, parse_quote!(::aldrin::core))
}

fn gen_deserialize(input: DeriveInput, krate: Path) -> Result<TokenStream> {
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
    StructData::new(input, named, fields, krate)?.gen_deserialize()
}

impl StructData<'_> {
    fn gen_deserialize(&self) -> Result<TokenStream> {
        let name = self.name();
        let lifetimes = self.lifetimes();
        let ty = quote! { #name<#(#lifetimes),*> };
        let krate = self.krate();
        let lifetimes = self.lifetimes();
        let field_vars = self.fields().iter().filter_map(FieldData::deserialize_var);

        let fields = self
            .fields()
            .iter()
            .map(|field| field.gen_deserialize(krate));

        let field_finish = self
            .fields()
            .iter()
            .map(|field| field.deserialize_finish(krate));

        Ok(quote! {
            #[automatically_derived]
            impl<#(#lifetimes),*> #krate::Deserialize<Self> for #ty {
                fn deserialize(
                    deserializer: #krate::Deserializer,
                ) -> ::std::result::Result<Self, #krate::DeserializeError> {
                    let mut _deserializer = deserializer.deserialize_struct()?;

                    #(#field_vars)*

                    while !_deserializer.is_empty() {
                        let _deserializer = _deserializer.deserialize()?;

                        match _deserializer.id() {
                            #(#fields)*
                            _ => _deserializer.skip()?,
                        }
                    }

                    _deserializer.finish_with(|_fallback| {
                        Ok(Self {
                            #(#field_finish),*
                        })
                    })
                }
            }
        })
    }
}

impl FieldData<'_> {
    fn gen_deserialize(&self, krate: &Path) -> TokenStream {
        if self.is_fallback() {
            quote! {
                _ => _deserializer.add_to_unknown_fields()?,
            }
        } else if self.is_optional() {
            let id = self.id();
            let var = self.var();
            let ty = self.ty();

            quote! {
                #id => #var = _deserializer.deserialize::<#krate::tags::As<#ty>, _>()?,
            }
        } else {
            let id = self.id();
            let var = self.var();
            let ty = self.ty();

            quote! {
                #id => {
                    #var = _deserializer
                        .deserialize::<#krate::tags::As<#ty>, _>()
                        .map(::std::option::Option::Some)?;
                }
            }
        }
    }

    fn deserialize_var(&self) -> Option<TokenStream> {
        if self.is_fallback() {
            None
        } else {
            let var = self.var();

            Some(quote! {
                let mut #var = ::std::option::Option::None;
            })
        }
    }

    fn deserialize_finish(&self, krate: &Path) -> TokenStream {
        let name = self.name();

        if self.is_fallback() {
            quote! { #name: _fallback.into() }
        } else if self.is_optional() {
            let var = self.var();

            quote! { #name: #var }
        } else {
            let var = self.var();

            quote! { #name: #var.ok_or(#krate::DeserializeError::InvalidSerialization)? }
        }
    }
}

fn gen_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Token![,]>,
    krate: Path,
) -> Result<TokenStream> {
    EnumData::new(input, variants, krate)?.gen_deserialize()
}

impl EnumData<'_> {
    fn gen_deserialize(&self) -> Result<TokenStream> {
        let name = self.name();
        let lifetimes = self.lifetimes();
        let ty = quote! { #name<#(#lifetimes),*> };
        let krate = self.krate();
        let lifetimes = self.lifetimes();
        let variants = self.variants().iter().map(|var| var.gen_deserialize(krate));

        Ok(quote! {
            #[automatically_derived]
            impl<#(#lifetimes),*> #krate::Deserialize<Self> for #ty {
                fn deserialize(
                    deserializer: #krate::Deserializer,
                ) -> ::std::result::Result<Self, #krate::DeserializeError> {
                    let deserializer = deserializer.deserialize_enum()?;

                    match deserializer.variant() {
                        #(#variants)*
                        _ => {
                            ::std::result::Result::Err(
                                #krate::DeserializeError::InvalidSerialization,
                            )
                        }
                    }
                }
            }
        })
    }
}

impl VariantData<'_> {
    fn gen_deserialize(&self, krate: &Path) -> TokenStream {
        let name = self.name();

        if self.is_fallback() {
            quote! {
                _ => {
                    deserializer
                        .into_unknown_variant()
                        .map(|fallback| Self::#name(fallback.into()))
                }
            }
        } else if let Some(ty) = self.ty() {
            let id = self.id();

            quote! {
                #id => deserializer.deserialize::<#krate::tags::As<#ty>, _>().map(Self::#name),
            }
        } else {
            let id = self.id();

            quote! {
                #id => deserializer.deserialize_unit().map(|()| Self::#name {}),
            }
        }
    }
}
