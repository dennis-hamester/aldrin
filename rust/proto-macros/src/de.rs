use crate::options::{ItemOptions, Options};
use proc_macro2::TokenStream;
use syn::punctuated::Punctuated;
use syn::{
    Data, DeriveInput, Error, Field, Fields, GenericParam, Generics, Result, Token, Variant,
};

pub fn gen_deserialize(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs)?;
    let name = &input.ident;
    let krate = options.krate();

    let body = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => gen_struct(&fields.named, &options, true)?,
            Fields::Unnamed(fields) => gen_struct(&fields.unnamed, &options, false)?,
            Fields::Unit => gen_unit_struct(&options),
        },

        Data::Enum(data) => gen_enum(&data.variants, &options)?,

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
        impl #impl_generics #krate::Deserialize for #name #ty_generics #where_clause {
            fn deserialize(
                deserializer: #krate::Deserializer,
            ) -> ::std::result::Result<Self, #krate::DeserializeError> {
                #body
            }
        }
    })
}

fn add_trait_bounds(mut generics: Generics, options: &Options) -> Generics {
    let krate = options.krate();

    let predicates = &mut generics
        .where_clause
        .get_or_insert_with(|| syn::parse_quote!(where))
        .predicates;

    if let Some(de_bounds) = options.de_bounds() {
        predicates.extend(de_bounds.into_iter().cloned());
    } else {
        for param in &mut generics.params {
            if let GenericParam::Type(type_param) = param {
                let ident = &type_param.ident;
                predicates.push(syn::parse_quote!(#ident: #krate::Deserialize));
            }
        }
    }

    generics
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
                let field_ident = quote::format_ident!("field{index}");
                next_id = item_options.id() + 1;
                Ok((field, item_options, field_ident))
            })
            .collect::<Result<Vec<_>>>()?
    };

    let field_vars = fields.iter().map(|(_, _, field_ident)| {
        quote::quote! {
            let mut #field_ident = ::std::option::Option::None;
        }
    });

    let match_arms = fields.iter().map(|(_, item_options, field_ident)| {
        let id = item_options.id();

        quote::quote! {
            #id => #field_ident = deserializer.deserialize().map(::std::option::Option::Some)?,
        }
    });

    let ok_expr = if named {
        let field_inits = fields.iter().map(|(field, _, field_ident)| {
            let ident = field.ident.as_ref().unwrap();
            quote::quote! {
                #ident: #field_ident.ok_or(#krate::DeserializeError::InvalidSerialization)?
            }
        });

        quote::quote! {
            Self { #(#field_inits),* }
        }
    } else {
        let field_inits = fields.iter().map(|(_, _, field_ident)| {
            quote::quote! {
                #field_ident.ok_or(#krate::DeserializeError::InvalidSerialization)?
            }
        });

        quote::quote! {
            Self(#(#field_inits),*)
        }
    };

    Ok(quote::quote! {
        let mut deserializer = deserializer.deserialize_struct()?;
        #(#field_vars)*

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;

            match deserializer.id() {
                #(#match_arms)*
                _ => deserializer.skip()?,
            }
        }

        Ok(#ok_expr)
    })
}

fn gen_unit_struct(options: &Options) -> TokenStream {
    let krate = options.krate();

    quote::quote! {
        if deserializer.deserialize_struct()?.has_more_fields() {
            Err(#krate::DeserializeError::InvalidSerialization)
        } else {
            Ok(Self)
        }
    }
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
                (true, _) => quote::quote! {
                    deserializer.deserialize().map(Self::#ident)
                },

                (false, true) => quote::quote! {
                    deserializer.deserialize().map(|()| Self::#ident)
                },

                (false, false) => quote::quote! {
                    deserializer.deserialize().map(|()| Self::#ident())
                },
            };

            quote::quote! {
                #id => #rhs,
            }
        });

    Ok(quote::quote! {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.variant() {
            #(#match_arms)*
            _ => Err(#krate::DeserializeError::InvalidSerialization),
        }
    })
}
