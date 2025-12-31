use super::StructData;
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Error, Field, Fields, Path, Result, Token, parse_quote};

pub(crate) fn gen_deserialize_key_from_core(input: DeriveInput) -> Result<TokenStream> {
    gen_deserialize_key(input, parse_quote!(::aldrin_core))
}

pub(crate) fn gen_deserialize_key_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    gen_deserialize_key(input, parse_quote!(::aldrin::core))
}

fn gen_deserialize_key(input: DeriveInput, krate: Path) -> Result<TokenStream> {
    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => gen_struct(&input, true, &fields.named, krate),
            Fields::Unnamed(ref fields) => gen_struct(&input, false, &fields.unnamed, krate),
            Fields::Unit => gen_struct(&input, false, &Punctuated::new(), krate),
        },

        Data::Enum(_) | Data::Union(_) => Err(Error::new_spanned(
            input,
            "`DeserializeKey` can only be derived for structs",
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
        Ok(struct_data.gen_deserialize_key())
    } else {
        Err(Error::new_spanned(
            input,
            "#[aldrin(newtype)] is required to derive `DeserializeKey`",
        ))
    }
}

impl StructData<'_> {
    fn gen_deserialize_key(&self) -> TokenStream {
        let name = self.name();
        let lifetimes = self.lifetimes();
        let ty = quote! { #name<#(#lifetimes),*> };
        let krate = self.krate();
        let lifetimes = self.lifetimes();
        let field = &self.fields()[0];
        let field_name = field.name();
        let field_var = field.var();
        let field_ty = field.ty();

        quote! {
            #[automatically_derived]
            impl<#(#lifetimes),*> #krate::DeserializeKey<Self> for #ty {
                fn try_from_key(
                    key: <<Self as #krate::tags::KeyTag>::Impl as #krate::tags::KeyTagImpl>::Key<
                        '_,
                    >,
                ) -> ::std::result::Result<Self, #krate::DeserializeError> {
                    #krate::DeserializeKey::<#krate::tags::AsKey<#field_ty>>::try_from_key(
                        key,
                    ).map(|#field_var| Self { #field_name: #field_var })
                }
            }
        }
    }
}
