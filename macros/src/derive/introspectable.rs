use super::{ItemOptions, Options};
use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::{parse_quote, Data, DeriveInput, Error, Field, Fields, Result, Token, Variant};

pub(crate) fn gen_introspectable_from_core(input: DeriveInput) -> Result<TokenStream> {
    let is_struct = matches!(input.data, Data::Struct(_));

    let options = Options::new(
        &input.ident,
        &input.attrs,
        parse_quote!(::aldrin_core),
        is_struct,
    )?;

    gen_introspectable(input, options)
}

pub(crate) fn gen_introspectable_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    let is_struct = matches!(input.data, Data::Struct(_));

    let options = Options::new(
        &input.ident,
        &input.attrs,
        parse_quote!(::aldrin::core),
        is_struct,
    )?;

    gen_introspectable(input, options)
}

fn gen_introspectable(input: DeriveInput, options: Options) -> Result<TokenStream> {
    super::ensure_no_type_generics(&input.generics)?;

    let schema = options.schema().ok_or_else(|| {
        Error::new_spanned(
            &input,
            "aldrin(schema = \"...\") must be set to derive Introspectable",
        )
    })?;

    let ident = &input.ident;
    let name = ident.unraw().to_string();
    let krate = options.krate();

    let (layout, add_references) = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => gen_struct(&fields.named, &name, &options)?,
            Fields::Unnamed(fields) => gen_struct(&fields.unnamed, &name, &options)?,
            Fields::Unit => gen_struct(&Punctuated::new(), &name, &options)?,
        },

        Data::Enum(data) => gen_enum(&data.variants, &name, &options)?,

        Data::Union(_) => {
            return Err(Error::new_spanned(
                input.ident,
                "unions are not supported by Aldrin",
            ))
        }
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #krate::introspection::Introspectable for #ident #ty_generics #where_clause {
            fn layout() -> #krate::introspection::ir::LayoutIr {
                #layout
            }

            fn lexical_id() -> #krate::introspection::LexicalId {
                #krate::introspection::LexicalId::custom(#schema, #name)
            }

            fn add_references(references: &mut #krate::introspection::References) {
                #add_references
            }
        }
    })
}

fn gen_struct(
    fields: &Punctuated<Field, Token![,]>,
    name: &str,
    options: &Options,
) -> Result<(TokenStream, TokenStream)> {
    if options.newtype() {
        gen_newtype_struct(fields, name, options)
    } else {
        gen_regular_struct(fields, name, options)
    }
}

fn gen_regular_struct(
    fields: &Punctuated<Field, Token![,]>,
    name: &str,
    options: &Options,
) -> Result<(TokenStream, TokenStream)> {
    let krate = options.krate();
    let schema = options.schema().unwrap();

    let mut layout = Vec::new();
    let mut references = Vec::new();
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

            fallback = match field.ident {
                Some(ref ident) => Some(ident.unraw().to_string()),
                None => Some("fallback".to_owned()),
            };
        } else {
            if fallback.is_some() {
                return Err(Error::new_spanned(
                    field,
                    "fields after the fallback are not allowed",
                ));
            }

            next_id = item_options.id() + 1;

            let (field_layout, field_references) = gen_field(field, index, options, &item_options);

            layout.push(field_layout);
            references.push(field_references);
        }
    }

    let fallback = fallback.map(|ident| {
        quote! {
            .fallback(#krate::introspection::ir::StructFallbackIr::builder(#ident).finish())
        }
    });

    let layout = quote! {
        #krate::introspection::ir::StructIr::builder(#schema, #name)
            #(#layout)*
            #fallback
            .finish()
            .into()
    };

    let add_references = if fields.is_empty() {
        TokenStream::new()
    } else {
        let len = references.len();

        quote! {
            let types: [#krate::introspection::DynIntrospectable; #len] = [
                #(#references),*
            ];

            references.extend(types);
        }
    };

    Ok((layout, add_references))
}

fn gen_field(
    field: &Field,
    index: usize,
    options: &Options,
    item_options: &ItemOptions,
) -> (TokenStream, TokenStream) {
    let krate = options.krate();
    let id = item_options.id();
    let is_required = !item_options.is_optional();
    let field_type = &field.ty;

    let name = match field.ident {
        Some(ref name) => name.unraw().to_string(),
        None => format!("field{index}"),
    };

    let lexical_id = if is_required {
        quote! { <#field_type as #krate::introspection::Introspectable>::lexical_id() }
    } else {
        quote! { <#field_type as #krate::introspection::private::OptionHelper>::lexical_id() }
    };

    let layout = quote! {
        .field(
            #krate::introspection::ir::FieldIr::builder(#id, #name, #is_required, #lexical_id)
                .finish(),
        )
    };

    let references = if is_required {
        quote! { #krate::introspection::DynIntrospectable::new::<#field_type>() }
    } else {
        quote! {
            <#field_type as #krate::introspection::private::OptionHelper>::dyn_introspectable()
        }
    };

    (layout, references)
}

fn gen_newtype_struct(
    fields: &Punctuated<Field, Token![,]>,
    name: &str,
    options: &Options,
) -> Result<(TokenStream, TokenStream)> {
    let krate = options.krate();
    let schema = options.schema().unwrap();
    let field = &fields[0];
    let field_type = &field.ty;

    let layout = quote! {
        #krate::introspection::ir::NewtypeIr::builder(
            #schema,
            #name,
            <#field_type as #krate::introspection::Introspectable>::lexical_id(),
        )
        .finish()
        .into()
    };

    let add_references = quote! {
        references.add::<#field_type>();
    };

    Ok((layout, add_references))
}

fn gen_enum(
    variants: &Punctuated<Variant, Token![,]>,
    name: &str,
    options: &Options,
) -> Result<(TokenStream, TokenStream)> {
    let krate = options.krate();
    let schema = options.schema().unwrap();

    let mut layout: Vec<TokenStream> = Vec::new();
    let mut references: Vec<TokenStream> = Vec::new();
    let mut next_id = 0;
    let mut has_fallback = false;

    for variant in variants.into_iter() {
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

        let (var_layout, var_references) = gen_variant(variant, options, &item_options)?;

        layout.push(var_layout);

        if let Some(var_references) = var_references {
            references.push(var_references);
        }
    }

    let layout = quote! {
        #krate::introspection::ir::EnumIr::builder(#schema, #name)
            #(#layout)*
            .finish()
            .into()
    };

    let add_references = if references.is_empty() {
        TokenStream::new()
    } else {
        let len = references.len();

        quote! {
            let types: [#krate::introspection::DynIntrospectable; #len] = [
                #(#references),*
            ];

            references.extend(types);
        }
    };

    Ok((layout, add_references))
}

fn gen_variant(
    variant: &Variant,
    options: &Options,
    item_options: &ItemOptions,
) -> Result<(TokenStream, Option<TokenStream>)> {
    if item_options.is_optional() {
        return Err(Error::new_spanned(
            variant,
            "enum variants cannot be optional",
        ));
    }

    if item_options.is_fallback() {
        gen_fallback_variant(variant, options).map(|toks| (toks, None))
    } else {
        gen_regular_variant(variant, options, item_options)
    }
}

fn gen_regular_variant(
    variant: &Variant,
    options: &Options,
    item_options: &ItemOptions,
) -> Result<(TokenStream, Option<TokenStream>)> {
    let krate = options.krate();
    let id = item_options.id();
    let name = variant.ident.unraw().to_string();

    let (variant_type, references) = match variant.fields {
        Fields::Unnamed(ref fields) if fields.unnamed.is_empty() => (None, None),

        Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => {
            let var_type = &fields.unnamed[0].ty;

            (
                Some(var_type),
                Some(quote! { #krate::introspection::DynIntrospectable::new::<#var_type>() }),
            )
        }

        Fields::Unnamed(_) => {
            return Err(Error::new_spanned(
                variant,
                "tuple-like variants with more than 1 element are not supported by Aldrin",
            ));
        }

        Fields::Unit => (None, None),

        Fields::Named(_) => {
            return Err(Error::new_spanned(
                variant,
                "struct-like variants are not supported by Aldrin",
            ));
        }
    };

    let variant_type = variant_type.map(|ty| {
        quote! {
            .variant_type(<#ty as #krate::introspection::Introspectable>::lexical_id())
        }
    });

    let layout = quote! {
        .variant(
            #krate::introspection::ir::VariantIr::builder(#id, #name)
                #variant_type
                .finish(),
        )
    };

    Ok((layout, references))
}

fn gen_fallback_variant(variant: &Variant, options: &Options) -> Result<TokenStream> {
    let krate = options.krate();
    let name = variant.ident.unraw().to_string();

    match variant.fields {
        Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => Ok(quote! {
            .fallback(#krate::introspection::ir::EnumFallbackIr::builder(#name).finish())
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
