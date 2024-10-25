use super::{add_trait_bounds, Options};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{parse_quote, Data, DeriveInput, Error, Field, Fields, Index, Result, Token};

pub fn gen_serialize_key_from_core(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin_core))?;
    gen_serialize_key(input, options)
}

pub fn gen_serialize_key_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin::core))?;
    gen_serialize_key(input, options)
}

fn gen_serialize_key(input: DeriveInput, options: Options) -> Result<TokenStream> {
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
                "SerializeKey cannot be derived for enums",
            ))
        }

        Data::Union(_) => {
            return Err(Error::new_spanned(
                input.ident,
                "SerializeKey cannot be derived for unions",
            ))
        }
    };

    let generics = add_trait_bounds(
        input.generics,
        &parse_quote!(#krate::SerializeKey),
        options.ser_key_bounds(),
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #krate::SerializeKey for #name #ty_generics #where_clause {
            #body
        }
    })
}

fn gen_struct(fields: &Punctuated<Field, Token![,]>, options: &Options) -> Result<TokenStream> {
    if fields.len() != 1 {
        return Err(Error::new_spanned(
            fields,
            "SerializeKey can only be derived for structs with exactly one field",
        ));
    }

    let krate = options.krate();
    let field = &fields[0];
    let ty = &field.ty;

    let acc = if let Some(ref ident) = field.ident {
        ident.to_token_stream()
    } else {
        Index::from(0).into_token_stream()
    };

    Ok(quote! {
        type Impl<'_impl>
            = <#ty as #krate::SerializeKey>::Impl<'_impl>
        where
            Self: '_impl;

        fn as_impl(&self) -> Self::Impl<'_> {
            #krate::SerializeKey::as_impl(&self.#acc)
        }
    })
}
