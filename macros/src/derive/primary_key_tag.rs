use super::StructData;
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{parse_quote, Data, DeriveInput, Error, Field, Fields, Path, Result, Token};

pub fn gen_primary_key_tag_from_core(input: DeriveInput) -> Result<TokenStream> {
    gen_primary_key_tag(input, parse_quote!(::aldrin_core))
}

pub fn gen_primary_key_tag_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    gen_primary_key_tag(input, parse_quote!(::aldrin::core))
}

fn gen_primary_key_tag(input: DeriveInput, krate: Path) -> Result<TokenStream> {
    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => gen_struct(&input, true, &fields.named, krate),
            Fields::Unnamed(ref fields) => gen_struct(&input, false, &fields.unnamed, krate),
            Fields::Unit => gen_struct(&input, false, &Punctuated::new(), krate),
        },

        Data::Enum(_) | Data::Union(_) => Err(Error::new_spanned(
            input,
            "`PrimaryKeyTag` can only be derived for structs",
        )),
    }
}

fn gen_struct(
    input: &DeriveInput,
    named: bool,
    fields: &Punctuated<Field, Token![,]>,
    krate: Path,
) -> Result<TokenStream> {
    let struct_data = StructData::new(input, named, fields, krate)?;

    if struct_data.is_newtype() {
        Ok(struct_data.gen_primary_key_tag())
    } else {
        Err(Error::new_spanned(
            input,
            "#[aldrin(newtype)] is required to derive `PrimaryKeyTag`",
        ))
    }
}

impl StructData<'_> {
    fn gen_primary_key_tag(&self) -> TokenStream {
        let name = self.name();
        let lifetimes = self.lifetimes();
        let ty = quote! { #name<#(#lifetimes),*> };
        let krate = self.krate();
        let lifetimes = self.lifetimes();

        quote! {
            #[automatically_derived]
            impl<#(#lifetimes),*> #krate::tags::PrimaryKeyTag for #ty {
                type KeyTag = Self;
            }
        }
    }
}
