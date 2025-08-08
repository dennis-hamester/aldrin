use super::{ensure_no_type_generics, Options};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Result};

pub(crate) fn gen_primary_tag_from_core(input: DeriveInput) -> Result<TokenStream> {
    let is_struct = matches!(input.data, Data::Struct(_));

    let options = Options::new(
        &input.ident,
        &input.attrs,
        parse_quote!(::aldrin_core),
        is_struct,
    )?;

    gen_primary_tag(input, options)
}

pub(crate) fn gen_primary_tag_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    let is_struct = matches!(input.data, Data::Struct(_));

    let options = Options::new(
        &input.ident,
        &input.attrs,
        parse_quote!(::aldrin::core),
        is_struct,
    )?;

    gen_primary_tag(input, options)
}

fn gen_primary_tag(input: DeriveInput, options: Options) -> Result<TokenStream> {
    let name = &input.ident;
    let krate = options.krate();

    ensure_no_type_generics(&input.generics)?;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #krate::tags::PrimaryTag for #name #ty_generics #where_clause {
            type Tag = Self;
        }
    })
}
