use super::{add_trait_bounds, ItemOptions, Options};
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{parse_quote, Data, DeriveInput, Error, Field, Fields, Index, Result, Token, Variant};

pub fn gen_serialize_from_core(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin_core))?;
    gen_serialize(input, options)
}

pub fn gen_serialize_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin::core))?;
    gen_serialize(input, options)
}

fn gen_serialize(input: DeriveInput, options: Options) -> Result<TokenStream> {
    let name = &input.ident;
    let krate = options.krate();

    let body = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => gen_struct(&fields.named)?,
            Fields::Unnamed(fields) => gen_struct(&fields.unnamed)?,
            Fields::Unit => gen_struct(&Punctuated::new())?,
        },

        Data::Enum(data) => gen_enum(&data.variants)?,

        Data::Union(_) => {
            return Err(Error::new_spanned(
                input.ident,
                "unions are not supported by Aldrin",
            ))
        }
    };

    let generics = add_trait_bounds(
        input.generics,
        &parse_quote!(#krate::Serialize),
        options.ser_bounds(),
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #krate::Serialize for #name #ty_generics #where_clause {
            fn serialize(
                &self,
                serializer: #krate::Serializer
            ) -> ::std::result::Result<(), #krate::SerializeError> {
                #body
            }
        }
    })
}

fn gen_struct(fields: &Punctuated<Field, Token![,]>) -> Result<TokenStream> {
    let mut num_required_fields = 0usize;
    let mut num_optional_fields = Vec::new();
    let mut body = Vec::new();
    let mut next_id = 0;
    let mut fallback = None;

    for (index, field) in fields.into_iter().enumerate() {
        let item_options = ItemOptions::new(&field.attrs, next_id)?;

        if item_options.is_fallback() {
            if fallback.is_some() {
                return Err(Error::new_spanned(
                    field,
                    "only one field can be marked fallback",
                ));
            }

            if item_options.is_optional() {
                return Err(Error::new_spanned(
                    field,
                    "fields cannot be marked both optional and fallback",
                ));
            }

            fallback = Some((index, field));
        } else {
            if fallback.is_some() {
                return Err(Error::new_spanned(
                    field,
                    "fields after the fallback are not allowed",
                ));
            }

            next_id = item_options.id() + 1;
            let (serialize, optional) = gen_field(field, index, &item_options)?;
            body.push(serialize);

            if let Some(optional) = optional {
                num_optional_fields.push(optional);
            } else {
                num_required_fields += 1;
            }
        }
    }

    let (num_var, num_ref) = match (num_required_fields, num_optional_fields.is_empty()) {
        (0, true) => (None, quote! { 0 }),

        (0, false) => (
            Some(quote! { let num = #(#num_optional_fields)+*; }),
            quote! { num },
        ),

        (num_required_fields, true) => (None, quote! { #num_required_fields }),

        (num_required_fields, false) => (
            Some(quote! { let num = #num_required_fields + #(#num_optional_fields)+*; }),
            quote! { num },
        ),
    };

    let serializer = match fallback {
        Some((index, field)) => match field.ident {
            Some(ref ident) => quote! {
                let mut serializer =
                    serializer.serialize_struct_with_unknown_fields(#num_ref, &self.#ident)?;
            },

            None => {
                let index = Index::from(index);

                quote! {
                    let mut serializer =
                        serializer.serialize_struct_with_unknown_fields(#num_ref, &self.#index)?;
                }
            }
        },

        None => quote! { let mut serializer = serializer.serialize_struct(#num_ref)?; },
    };

    Ok(quote! {
        #num_var
        #serializer
        #(#body)*
        serializer.finish()
    })
}

fn gen_field(
    field: &Field,
    index: usize,
    item_options: &ItemOptions,
) -> Result<(TokenStream, Option<TokenStream>)> {
    let id = item_options.id();

    let (serialize, optional) = match (field.ident.as_ref(), item_options.is_optional()) {
        (Some(ident), true) => {
            let serialize = quote! {
                if ::std::option::Option::is_some(&self.#ident) {
                    serializer.serialize_field(#id, &self.#ident)?;
                }
            };

            let optional = Some(quote! { (::std::option::Option::is_some(&self.#ident) as usize) });
            (serialize, optional)
        }

        (Some(ident), false) => {
            let serialize = quote! { serializer.serialize_field(#id, &self.#ident)?; };
            (serialize, None)
        }

        (None, true) => {
            let index = Index::from(index);

            let serialize = quote! {
                if ::std::option::Option::is_some(&self.#index) {
                    serializer.serialize_field(#id, &self.#index)?;
                }
            };

            let optional = Some(quote! { (::std::option::Option::is_some(&self.#index) as usize) });
            (serialize, optional)
        }

        (None, false) => {
            let index = Index::from(index);
            let serialize = quote! { serializer.serialize_field(#id, &self.#index)?; };
            (serialize, None)
        }
    };

    Ok((serialize, optional))
}

fn gen_enum(variants: &Punctuated<Variant, Token![,]>) -> Result<TokenStream> {
    let body = {
        let mut next_id = 0;
        let mut has_fallback = false;

        variants
            .into_iter()
            .map(|variant| {
                let item_options = ItemOptions::new(&variant.attrs, next_id)?;

                if item_options.is_fallback() {
                    if has_fallback {
                        return Err(Error::new_spanned(
                            variant,
                            "only one variant can be marked fallback",
                        ));
                    }

                    has_fallback = true;
                } else if has_fallback {
                    return Err(Error::new_spanned(
                        variant,
                        "variants after the fallback are not allowed",
                    ));
                }

                next_id = item_options.id() + 1;
                gen_variant(variant, &item_options)
            })
            .collect::<Result<Vec<_>>>()?
    };

    Ok(quote! {
        match *self {
            #(#body),*
        }
    })
}

fn gen_variant(variant: &Variant, item_options: &ItemOptions) -> Result<TokenStream> {
    if item_options.is_optional() {
        return Err(Error::new_spanned(
            variant,
            "enum variants cannot be optional",
        ));
    }

    if item_options.is_fallback() {
        gen_fallback_variant(variant)
    } else {
        gen_regular_variant(variant, item_options)
    }
}

fn gen_regular_variant(variant: &Variant, item_options: &ItemOptions) -> Result<TokenStream> {
    let ident = &variant.ident;
    let id = item_options.id();

    match variant.fields {
        Fields::Unnamed(ref fields) if fields.unnamed.is_empty() => Ok(quote! {
            Self::#ident() => serializer.serialize_enum(#id, &())
        }),

        Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => Ok(quote! {
            Self::#ident(ref val) => serializer.serialize_enum(#id, val)
        }),

        Fields::Unnamed(_) => Err(Error::new_spanned(
            variant,
            "tuple-like variants with more than 1 element are not supported by Aldrin",
        )),

        Fields::Unit => Ok(quote! { Self::#ident => serializer.serialize_enum(#id, &()) }),

        Fields::Named(_) => Err(Error::new_spanned(
            variant,
            "struct-like variants are not supported by Aldrin",
        )),
    }
}

fn gen_fallback_variant(variant: &Variant) -> Result<TokenStream> {
    let ident = &variant.ident;

    match variant.fields {
        Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => Ok(quote! {
            Self::#ident(ref val) => serializer.serialize_unknown_variant(val)
        }),

        Fields::Unnamed(_) | Fields::Unit => Err(Error::new_spanned(
            variant,
            "the fallback variant must have exactly 1 element",
        )),

        Fields::Named(_) => Err(Error::new_spanned(
            variant,
            "struct-like variants are not supported by Aldrin",
        )),
    }
}
