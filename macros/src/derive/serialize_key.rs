use super::{FieldData, StructData};
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    parse_quote, Data, DeriveInput, Error, Field, Fields, GenericParam, Path, Result, Token,
};

pub fn gen_serialize_key_from_core(input: DeriveInput) -> Result<TokenStream> {
    gen_serialize_key(input, parse_quote!(::aldrin_core))
}

pub fn gen_serialize_key_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    gen_serialize_key(input, parse_quote!(::aldrin::core))
}

fn gen_serialize_key(input: DeriveInput, krate: Path) -> Result<TokenStream> {
    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => gen_struct(&input, true, &fields.named, krate),
            Fields::Unnamed(ref fields) => gen_struct(&input, false, &fields.unnamed, krate),
            Fields::Unit => gen_struct(&input, false, &Punctuated::new(), krate),
        },

        Data::Enum(_) | Data::Union(_) => Err(Error::new_spanned(
            input,
            "`SerializeKey` can only be derived for structs",
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
        Ok(struct_data.gen_serialize_key())
    } else {
        Err(Error::new_spanned(
            input,
            "#[aldrin(newtype)] is required to derive `SerializeKey`",
        ))
    }
}

impl StructData<'_> {
    fn gen_serialize_key(&self) -> TokenStream {
        let for_self = self.gen_serialize_key_for_self();
        let for_ref_type = self.gen_serialize_key_for_ref_type();

        quote! {
            #for_self
            #for_ref_type
        }
    }

    fn gen_serialize_key_for_self(&self) -> TokenStream {
        let name = self.name();
        let lifetimes = self.lifetimes();
        let ty = quote! { #name<#(#lifetimes),*> };
        let krate = self.krate();
        let lifetimes = self.lifetimes();
        let field = &self.fields()[0];
        let field_name = field.name();
        let field_ty = field.ty();

        quote! {
            #[automatically_derived]
            impl<#(#lifetimes),*> #krate::SerializeKey<Self> for #ty {
                fn try_as_key(
                    &self,
                ) -> ::std::result::Result<
                    <<Self as #krate::tags::KeyTag>::Impl as #krate::tags::KeyTagImpl>::Key<'_>,
                    #krate::SerializeError,
                > {
                    #krate::SerializeKey::<#krate::tags::AsKey<#field_ty>>::try_as_key(
                        &self.#field_name,
                    )
                }
            }
        }
    }

    fn gen_serialize_key_for_ref_type(&self) -> Option<TokenStream> {
        let ref_type = self.ref_type()?;
        let name = self.name();
        let lifetimes = self.lifetimes();
        let ty = quote! { #name<#(#lifetimes),*> };
        let krate = self.krate();
        let ty_generics = self.ty_generics();
        let field = &self.fields()[0];
        let where_bound = field.serialize_key_where_bound_for_ref_type(krate);
        let field_name = field.name();

        let impl_generics = self
            .lifetimes()
            .iter()
            .map(|lt| GenericParam::Lifetime((*lt).clone()))
            .chain(
                self.ty_generics()
                    .map(|ty| GenericParam::Type(ty.clone().into())),
            );

        Some(quote! {
            #[automatically_derived]
            impl<#(#impl_generics),*> #krate::SerializeKey<#ty> for #ref_type<#(#ty_generics),*>
            where
                #where_bound,
            {
                fn try_as_key(
                    &self,
                ) -> ::std::result::Result<
                    <<#ty as #krate::tags::KeyTag>::Impl as #krate::tags::KeyTagImpl>::Key<'_>,
                    #krate::SerializeError,
                > {
                    self.#field_name.try_as_key()
                }
            }
        })
    }
}

impl FieldData<'_> {
    fn serialize_key_where_bound_for_ref_type(&self, krate: &Path) -> TokenStream {
        let ty_generic = self.ty_generic();
        let ty_tag = self.ty_tag();

        quote! { #ty_generic: #krate::SerializeKey<#krate::tags::AsKey<#ty_tag>> }
    }
}
