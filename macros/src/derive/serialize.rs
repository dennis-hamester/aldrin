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

    for (index, field) in fields.into_iter().enumerate() {
        let (serialize, id, optional) = gen_field(field, index, next_id)?;

        body.push(serialize);
        next_id = id + 1;

        if let Some(optional) = optional {
            num_optional_fields.push(optional);
        } else {
            num_required_fields += 1;
        }
    }

    match (num_required_fields, num_optional_fields.is_empty()) {
        (0, true) => Ok(quote! { serializer.serialize_struct(0)?.finish() }),

        (num_required_fields, true) => Ok(quote! {
            let mut serializer = serializer.serialize_struct(#num_required_fields)?;
            #(#body)*
            serializer.finish()
        }),

        (0, false) => Ok(quote! {
            let num_fields = #(#num_optional_fields)+*;
            let mut serializer = serializer.serialize_struct(num_fields)?;
            #(#body)*
            serializer.finish()
        }),

        (num_required_fields, false) => Ok(quote! {
            let num_fields = #num_required_fields + #(#num_optional_fields)+*;
            let mut serializer = serializer.serialize_struct(num_fields)?;
            #(#body)*
            serializer.finish()
        }),
    }
}

fn gen_field(
    field: &Field,
    index: usize,
    default_id: u32,
) -> Result<(TokenStream, u32, Option<TokenStream>)> {
    let item_options = ItemOptions::new(&field.attrs, default_id)?;
    let id = item_options.id();

    let (serialize, optional) = match (field.ident.as_ref(), item_options.is_optional()) {
        (Some(ident), true) => {
            let serialize = quote! {
                if ::std::option::Option::is_some(&self.#ident) {
                    serializer.serialize_field(#id, &self.#ident)?;
                }
            };

            let optional = Some(quote! {
                if ::std::option::Option::is_some(&self.#ident) { 1 } else { 0 }
            });

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

            let optional = Some(quote! {
                if ::std::option::Option::is_some(&self.#index) { 1 } else { 0 }
            });

            (serialize, optional)
        }

        (None, false) => {
            let index = Index::from(index);
            let serialize = quote! { serializer.serialize_field(#id, &self.#index)?; };
            (serialize, None)
        }
    };

    Ok((serialize, id, optional))
}

fn gen_enum(variants: &Punctuated<Variant, Token![,]>) -> Result<TokenStream> {
    let body = {
        let mut next_id = 0;

        variants
            .into_iter()
            .map(|variant| {
                let (tokens, id) = gen_variant(variant, next_id)?;
                next_id = id + 1;
                Ok(tokens)
            })
            .collect::<Result<Vec<_>>>()?
    };

    Ok(quote! {
        match *self {
            #(#body),*
        }
    })
}

fn gen_variant(variant: &Variant, default_id: u32) -> Result<(TokenStream, u32)> {
    let item_options = ItemOptions::new(&variant.attrs, default_id)?;

    if item_options.is_optional() {
        return Err(Error::new_spanned(
            variant,
            "enum variants cannot be optional",
        ));
    }

    let ident = &variant.ident;
    let id = item_options.id();

    let tokens = match variant.fields {
        Fields::Unnamed(ref fields) if fields.unnamed.is_empty() => quote! {
            Self::#ident() => serializer.serialize_enum(#id, &())
        },

        Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => quote! {
            Self::#ident(ref val) => serializer.serialize_enum(#id, val)
        },

        Fields::Unnamed(_) => {
            return Err(Error::new_spanned(
                variant,
                "tuple-like variants with more than 1 element are not supported by Aldrin",
            ))
        }

        Fields::Unit => quote! { Self::#ident => serializer.serialize_enum(#id, &()) },

        Fields::Named(_) => {
            return Err(Error::new_spanned(
                variant,
                "struct-like variants are not supported by Aldrin",
            ))
        }
    };

    Ok((tokens, id))
}
