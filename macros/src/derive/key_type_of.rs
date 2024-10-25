use super::{add_trait_bounds, Options};
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{parse_quote, Data, DeriveInput, Error, Field, Fields, Result, Token};

pub fn gen_key_type_of_from_core(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin_core))?;
    gen_key_type_of(input, options)
}

pub fn gen_key_type_of_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin::core))?;
    gen_key_type_of(input, options)
}

fn gen_key_type_of(input: DeriveInput, options: Options) -> Result<TokenStream> {
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
                "KeyTypeOf cannot be derived for enums",
            ))
        }

        Data::Union(_) => {
            return Err(Error::new_spanned(
                input.ident,
                "KeyTypeOf cannot be derived for unions",
            ))
        }
    };

    let generics = add_trait_bounds(
        input.generics,
        &parse_quote!(#krate::KeyTypeOf),
        options.key_ty_bounds(),
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #krate::introspection::KeyTypeOf for #name #ty_generics #where_clause {
            #body
        }
    })
}

fn gen_struct(fields: &Punctuated<Field, Token![,]>, options: &Options) -> Result<TokenStream> {
    if fields.len() != 1 {
        return Err(Error::new_spanned(
            fields,
            "KeyTypeOf can only be derived for structs with exactly one field",
        ));
    }

    let krate = options.krate();
    let ty = &fields[0].ty;

    Ok(quote! {
        const KEY_TYPE: #krate::introspection::KeyType =
            <#ty as #krate::introspection::KeyTypeOf>::KEY_TYPE;
    })
}
