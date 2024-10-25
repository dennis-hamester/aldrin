use super::{add_trait_bounds, ItemOptions, Options};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{parse_quote, Data, DeriveInput, Error, Field, Fields, Result, Token, Variant};

pub fn gen_deserialize_from_core(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin_core))?;
    gen_deserialize(input, options)
}

pub fn gen_deserialize_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin::core))?;
    gen_deserialize(input, options)
}

fn gen_deserialize(input: DeriveInput, options: Options) -> Result<TokenStream> {
    let name = &input.ident;
    let krate = options.krate();

    let body = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => gen_struct(&fields.named, &options, true)?,
            Fields::Unnamed(fields) => gen_struct(&fields.unnamed, &options, false)?,

            Fields::Unit => quote! {
                deserializer.deserialize_struct()?.skip()?;
                ::std::result::Result::Ok(Self)
            },
        },

        Data::Enum(data) => gen_enum(&data.variants, &options)?,

        Data::Union(_) => {
            return Err(Error::new_spanned(
                input.ident,
                "unions are not supported by Aldrin",
            ))
        }
    };

    let generics = add_trait_bounds(
        input.generics,
        &parse_quote!(#krate::Deserialize),
        options.de_bounds(),
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #krate::Deserialize for #name #ty_generics #where_clause {
            fn deserialize(
                deserializer: #krate::Deserializer,
            ) -> ::std::result::Result<Self, #krate::DeserializeError> {
                #body
            }
        }
    })
}

fn gen_struct(
    fields: &Punctuated<Field, Token![,]>,
    options: &Options,
    named: bool,
) -> Result<TokenStream> {
    let krate = options.krate();

    let fields = {
        let mut next_id = 0;

        fields
            .into_iter()
            .enumerate()
            .map(|(index, field)| {
                let item_options = ItemOptions::new(&field.attrs, next_id)?;
                let field_ident = format_ident!("field{index}");
                next_id = item_options.id() + 1;
                Ok((field, item_options, field_ident))
            })
            .collect::<Result<Vec<_>>>()?
    };

    let field_vars = fields.iter().map(|(_, _, field_ident)| {
        quote! { let mut #field_ident = ::std::option::Option::None; }
    });

    let match_arms = fields.iter().map(|(_, item_options, field_ident)| {
        let id = item_options.id();

        if item_options.is_optional() {
            quote! { #id => #field_ident = deserializer.deserialize()?, }
        } else {
            quote! {
                #id => #field_ident = deserializer.deserialize().map(::std::option::Option::Some)?,
            }
        }
    });

    let ok_expr = if named {
        let field_inits = fields.iter().map(|(field, item_options, field_ident)| {
            let ident = field.ident.as_ref().unwrap();

            if item_options.is_optional() {
                quote! { #ident: #field_ident }
            } else {
                quote! {
                    #ident: #field_ident.ok_or(#krate::DeserializeError::InvalidSerialization)?
                }
            }
        });

        quote! { Self { #(#field_inits),* } }
    } else {
        let field_inits = fields.iter().map(|(_, item_options, field_ident)| {
            if item_options.is_optional() {
                quote! { #field_ident }
            } else {
                quote! { #field_ident.ok_or(#krate::DeserializeError::InvalidSerialization)? }
            }
        });

        quote! { Self(#(#field_inits),*) }
    };

    Ok(quote! {
        let mut deserializer = deserializer.deserialize_struct()?;
        #(#field_vars)*

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            match deserializer.id() {
                #(#match_arms)*
                _ => deserializer.skip()?,
            }
        }

        deserializer.finish_with(|| ::std::result::Result::Ok(#ok_expr))
    })
}

fn gen_enum(variants: &Punctuated<Variant, Token![,]>, options: &Options) -> Result<TokenStream> {
    let krate = options.krate();

    let variants = {
        let mut next_id = 0;

        variants
            .into_iter()
            .map(|variant| {
                let item_options = ItemOptions::new(&variant.attrs, next_id)?;

                let (has_field, is_unit) = match variant.fields {
                    Fields::Unnamed(ref fields) if fields.unnamed.is_empty() => (false, false),
                    Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => (true, false),
                    Fields::Unit => (false, true),

                    Fields::Named(_) => {
                        return Err(Error::new_spanned(
                            variant,
                            "struct-like variants are not supported by Aldrin",
                        ))
                    }

                    Fields::Unnamed(_) => return Err(Error::new_spanned(
                        variant,
                        "tuple-like variants with more than 1 element are not supported by Aldrin",
                    )),
                };

                next_id = item_options.id() + 1;
                Ok((&variant.ident, item_options, has_field, is_unit))
            })
            .collect::<Result<Vec<_>>>()?
    };

    let match_arms = variants
        .iter()
        .map(|(ident, item_options, has_field, is_unit)| {
            let id = item_options.id();

            let rhs = match (has_field, is_unit) {
                (true, _) => quote! { deserializer.deserialize().map(Self::#ident) },
                (false, true) => quote! { deserializer.deserialize().map(|()| Self::#ident) },
                (false, false) => quote! { deserializer.deserialize().map(|()| Self::#ident()) },
            };

            quote! { #id => #rhs, }
        });

    Ok(quote! {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.variant() {
            #(#match_arms)*
            _ => ::std::result::Result::Err(#krate::DeserializeError::InvalidSerialization),
        }
    })
}
