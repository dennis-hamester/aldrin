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
    struct_fields: &Punctuated<Field, Token![,]>,
    options: &Options,
    named: bool,
) -> Result<TokenStream> {
    let krate = options.krate();
    let mut fields = Vec::new();
    let mut next_id = 0;
    let mut fallback = None;

    for field in struct_fields {
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

            fallback = Some(field);
        } else {
            if fallback.is_some() {
                return Err(Error::new_spanned(
                    field,
                    "fields after the fallback are not allowed",
                ));
            }

            next_id = item_options.id() + 1;

            let field_ident = if let Some(ref ident) = field.ident {
                ident.clone()
            } else {
                format_ident!("__field{}", item_options.id())
            };

            fields.push((field, item_options, field_ident));
        }
    }

    let field_vars = fields.iter().map(|(_, _, field_ident)| {
        quote! { let mut #field_ident = ::std::option::Option::None; }
    });

    let match_arms = fields.iter().map(|(_, item_options, field_ident)| {
        let id = item_options.id();

        if item_options.is_optional() {
            quote! { #id => #field_ident = __deserializer.deserialize()?, }
        } else {
            quote! {
                #id => {
                    #field_ident = __deserializer.deserialize().map(::std::option::Option::Some)?;
                }
            }
        }
    });

    let wildcard = if fallback.is_some() {
        quote! { _ => __deserializer.add_to_unknown_fields()?, }
    } else {
        quote! { _ => __deserializer.skip()?, }
    };

    let field_inits = fields.iter().map(|(field, item_options, field_ident)| {
        match (field.ident.as_ref(), item_options.is_optional()) {
            (Some(ref ident), false) => quote! {
                #ident: #ident.ok_or(#krate::DeserializeError::InvalidSerialization)?,
            },

            (Some(ref ident), true) => quote! { #ident, },

            (None, false) => quote! {
                #field_ident.ok_or(#krate::DeserializeError::InvalidSerialization)?,
            },

            (None, true) => quote! { #field_ident, },
        }
    });

    let finish = if let Some(fallback) = fallback {
        if let Some(ref ident) = fallback.ident {
            quote! {
                __deserializer.finish_with(|#ident| ::std::result::Result::Ok(Self {
                    #(#field_inits)*
                    #ident,
                }))
            }
        } else {
            quote! {
                __deserializer.finish_with(|__fallback| ::std::result::Result::Ok(Self(
                    #(#field_inits)*
                    __fallback,
                )))
            }
        }
    } else if named {
        quote! {
            __deserializer.finish_with(|_| ::std::result::Result::Ok(Self { #(#field_inits)* }))
        }
    } else {
        quote! {
            __deserializer.finish_with(|_| ::std::result::Result::Ok(Self(#(#field_inits)*)))
        }
    };

    Ok(quote! {
        let mut __deserializer = deserializer.deserialize_struct()?;
        #(#field_vars)*

        while __deserializer.has_more_fields() {
            let __deserializer = __deserializer.deserialize_field()?;

            match __deserializer.id() {
                #(#match_arms)*
                #wildcard
            }
        }

        #finish
    })
}

fn gen_enum(variants: &Punctuated<Variant, Token![,]>, options: &Options) -> Result<TokenStream> {
    let krate = options.krate();
    let mut next_id = 0;
    let mut has_fallback = false;

    let mut match_arms = Vec::new();
    for variant in variants {
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
        match_arms.push(gen_variant(variant, &item_options)?);
    }

    let catch_all = if has_fallback {
        TokenStream::new()
    } else {
        quote! { _ => ::std::result::Result::Err(#krate::DeserializeError::InvalidSerialization), }
    };

    Ok(quote! {
        let deserializer = deserializer.deserialize_enum()?;

        match deserializer.variant() {
            #(#match_arms)*
            #catch_all
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
            #id => deserializer.deserialize().map(|()| Self::#ident()),
        }),

        Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => Ok(quote! {
            #id => deserializer.deserialize().map(Self::#ident),
        }),

        Fields::Unnamed(_) => Err(Error::new_spanned(
            variant,
            "tuple-like variants with more than 1 element are not supported by Aldrin",
        )),

        Fields::Unit => Ok(quote! { #id => deserializer.deserialize().map(|()| Self::#ident), }),

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
            _ => deserializer.into_unknown_variant().map(Self::#ident),
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
