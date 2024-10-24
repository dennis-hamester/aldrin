use super::Options;
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    parse_quote, Data, DeriveInput, Error, Field, Fields, GenericParam, Generics, Result, Token,
};

pub fn gen_deserialize_key_from_core(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin_core))?;
    gen_deserialize_key(input, options)
}

pub fn gen_deserialize_key_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin::core))?;
    gen_deserialize_key(input, options)
}

fn gen_deserialize_key(input: DeriveInput, options: Options) -> Result<TokenStream> {
    let name = &input.ident;
    let krate = options.krate();

    let body = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => gen_struct(&fields.named, &options)?,
            Fields::Unnamed(fields) => gen_struct(&fields.unnamed, &options)?,
            Fields::Unit => gen_struct(&Punctuated::new(), &options)?,
        },

        Data::Enum(_) => {
            return Err(Error::new_spanned(
                input.ident,
                "DeserializeKey cannot be derived for enums",
            ))
        }

        Data::Union(_) => {
            return Err(Error::new_spanned(
                input.ident,
                "DeserializeKey cannot be derived for unions",
            ))
        }
    };

    let generics = add_trait_bounds(input.generics, &options);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #krate::DeserializeKey for #name #ty_generics #where_clause {
            #body
        }
    })
}

fn add_trait_bounds(mut generics: Generics, options: &Options) -> Generics {
    let krate = options.krate();

    let predicates = &mut generics
        .where_clause
        .get_or_insert_with(|| parse_quote!(where))
        .predicates;

    if let Some(bounds) = options.de_key_bounds() {
        predicates.extend(bounds.into_iter().cloned());
    } else {
        for param in &generics.params {
            if let GenericParam::Type(type_param) = param {
                let ident = &type_param.ident;
                predicates.push(parse_quote!(#ident: #krate::DeserializeKey));
            }
        }
    }

    generics
}

fn gen_struct(fields: &Punctuated<Field, Token![,]>, options: &Options) -> Result<TokenStream> {
    if fields.len() != 1 {
        return Err(Error::new_spanned(
            fields,
            "DeserializeKey can only be derived for structs with exactly one field",
        ));
    }

    let krate = options.krate();
    let field = &fields[0];
    let ty = &field.ty;

    let ctor = if let Some(ref ident) = field.ident {
        quote! { { #ident: key } }
    } else {
        quote! { (key) }
    };

    Ok(quote! {
        type Impl = <#ty as #krate::DeserializeKey>::Impl;

        fn try_from_impl(key: Self::Impl) -> ::std::result::Result<Self, #krate::DeserializeError> {
            let key = <#ty as #krate::DeserializeKey>::try_from_impl(key)?;
            ::std::result::Result::Ok(Self #ctor)
        }
    })
}
