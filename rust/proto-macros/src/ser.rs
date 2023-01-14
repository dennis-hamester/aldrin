use crate::options::{ItemOptions, Options};
use proc_macro2::TokenStream;
use syn::punctuated::Punctuated;
use syn::{
    Data, DeriveInput, Error, Field, Fields, GenericParam, Generics, Index, Result, Token, Variant,
};

pub fn gen_serialize(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs)?;
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

    let generics = add_trait_bounds(input.generics, &options);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote::quote! {
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

fn add_trait_bounds(mut generics: Generics, options: &Options) -> Generics {
    let krate = options.krate();

    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(syn::parse_quote!(#krate::Serialize));
        }
    }

    generics
}

fn gen_struct(fields: &Punctuated<Field, Token![,]>) -> Result<TokenStream> {
    let num_fields = fields.len();

    let body = {
        let mut next_id = 0;

        fields
            .into_iter()
            .enumerate()
            .map(|(index, field)| {
                let (tokens, id) = gen_field(field, index, next_id)?;
                next_id = id + 1;
                Ok(tokens)
            })
            .collect::<Result<Vec<_>>>()?
    };

    Ok(quote::quote! {
        let mut serializer = serializer.serialize_struct(#num_fields)?;
        #(#body)*
        serializer.finish()
    })
}

fn gen_field(field: &Field, index: usize, default_id: u32) -> Result<(TokenStream, u32)> {
    let item_options = ItemOptions::new(&field.attrs, default_id)?;
    let id = item_options.id();

    let tokens = if let Some(ref ident) = field.ident {
        quote::quote! {
            serializer.serialize_field(#id, &self.#ident)?;
        }
    } else {
        let index = Index::from(index);
        quote::quote! {
            serializer.serialize_field(#id, &self.#index)?;
        }
    };

    Ok((tokens, id))
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

    Ok(quote::quote! {
        match *self {
            #(#body),*
        }
    })
}

fn gen_variant(variant: &Variant, default_id: u32) -> Result<(TokenStream, u32)> {
    let item_options = ItemOptions::new(&variant.attrs, default_id)?;
    let ident = &variant.ident;
    let id = item_options.id();

    let tokens = match variant.fields {
        Fields::Unnamed(ref fields) if fields.unnamed.is_empty() => quote::quote! {
            Self::#ident() => serializer.serialize_enum(#id, &())
        },

        Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => quote::quote! {
            Self::#ident(ref field) => serializer.serialize_enum(#id, field)
        },

        Fields::Unit => quote::quote! {
            Self::#ident => serializer.serialize_enum(#id, &())
        },

        Fields::Named(_) => {
            return Err(Error::new_spanned(
                variant,
                "struct-like variants are not supported by Aldrin",
            ))
        }

        Fields::Unnamed(_) => {
            return Err(Error::new_spanned(
                variant,
                "tuple-like variants with more than 1 element are not supported by Aldrin",
            ))
        }
    };

    Ok((tokens, id))
}
