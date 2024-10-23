use super::{add_trait_bounds, Options};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, DeriveInput, Result};

pub fn gen_as_serialize_arg_from_core(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin_core))?;
    gen_as_serialize_arg(input, options)
}

pub fn gen_as_serialize_arg_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    let options = Options::new(&input.attrs, parse_quote!(::aldrin::core))?;
    gen_as_serialize_arg(input, options)
}

fn gen_as_serialize_arg(input: DeriveInput, options: Options) -> Result<TokenStream> {
    let name = &input.ident;
    let krate = options.krate();

    let generics = add_trait_bounds(
        input.generics,
        &parse_quote!(#krate::AsSerializeArg),
        options.ser_bounds(),
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #krate::AsSerializeArg for #name #ty_generics #where_clause {
            type SerializeArg<'_arg>
                = &'_arg Self
            where
                Self: '_arg;

            fn as_serialize_arg<'_arg>(&'_arg self) -> Self::SerializeArg<'_arg>
            where
                Self: '_arg,
            {
                self
            }
        }
    })
}
