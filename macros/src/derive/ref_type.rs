use super::{EnumData, FieldData, StructData, VariantData};
use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::ptr;
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::{
    parse_quote, Data, DeriveInput, Error, Field, Fields, Ident, Path, Result, Token, Variant,
};

pub fn gen_ref_type_from_core(input: DeriveInput) -> Result<TokenStream> {
    gen_ref_type(input, parse_quote!(::aldrin_core))
}

pub fn gen_ref_type_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    gen_ref_type(input, parse_quote!(::aldrin::core))
}

fn gen_ref_type(input: DeriveInput, krate: Path) -> Result<TokenStream> {
    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => gen_struct(&input, true, &fields.named, krate),
            Fields::Unnamed(ref fields) => gen_struct(&input, false, &fields.unnamed, krate),
            Fields::Unit => gen_struct(&input, false, &Punctuated::new(), krate),
        },

        Data::Enum(ref data) => gen_enum(&input, &data.variants, krate),

        Data::Union(_) => Err(Error::new_spanned(
            input.ident,
            "unions are not supported by Aldrin",
        )),
    }
}

fn gen_struct(
    input: &DeriveInput,
    named: bool,
    fields: &Punctuated<Field, Token![,]>,
    krate: Path,
) -> Result<TokenStream> {
    StructData::new(input, named, fields, krate)?.gen_ref_type()
}

impl StructData<'_> {
    fn gen_ref_type(&self) -> Result<TokenStream> {
        let ref_type = self.ref_type().ok_or_else(|| {
            Error::new(
                Span::call_site(),
                "#[aldrin(ref_type = ...)] is required to derive `RefType`",
            )
        })?;

        let vis = self.vis();
        let ty_generics = self.ty_generics();
        let fields = self.fields().iter().map(FieldData::gen_ref_type);

        if self.fields().is_empty() {
            Ok(quote! {
                #[automatically_derived]
                #[derive(::std::fmt::Debug, ::std::marker::Copy, ::std::clone::Clone)]
                #vis struct #ref_type<#(#ty_generics),*>;
            })
        } else if self.is_named() {
            Ok(quote! {
                #[automatically_derived]
                #[derive(::std::fmt::Debug, ::std::marker::Copy, ::std::clone::Clone)]
                #vis struct #ref_type<#(#ty_generics),*> {
                    #(#fields),*
                }
            })
        } else {
            Ok(quote! {
                #[automatically_derived]
                #[derive(::std::fmt::Debug, ::std::marker::Copy, ::std::clone::Clone)]
                #vis struct #ref_type<#(#ty_generics),*>(#(#fields),*);
            })
        }
    }
}

impl FieldData<'_> {
    fn gen_ref_type(&self) -> TokenStream {
        let ty_generic = self.ty_generic();

        if self.is_named() {
            let name = self.name();
            quote! { pub #name: #ty_generic }
        } else {
            quote! { pub #ty_generic }
        }
    }
}

fn gen_enum(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Token![,]>,
    krate: Path,
) -> Result<TokenStream> {
    EnumData::new(input, variants, krate)?.gen_ref_type()
}

impl EnumData<'_> {
    fn gen_ref_type(&self) -> Result<TokenStream> {
        let ref_type = self.ref_type().ok_or_else(|| {
            Error::new(
                Span::call_site(),
                "#[aldrin(ref_type = ...)] is required to derive `RefType`",
            )
        })?;

        let vis = self.vis();
        let ty_generics = self.ty_generics();
        let variants = self.variants().iter().map(VariantData::gen_ref_type);
        let ctors = self.ctors();

        Ok(quote! {
            #[automatically_derived]
            #[derive(::std::fmt::Debug, ::std::marker::Copy, ::std::clone::Clone)]
            #vis enum #ref_type<#(#ty_generics),*> {
                #(#variants),*
            }

            #(#ctors)*
        })
    }

    fn ctors(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.variants()
            .iter()
            .map(|var| var.ctor(self.ref_type().unwrap(), self.variants()))
    }
}

impl VariantData<'_> {
    fn gen_ref_type(&self) -> TokenStream {
        let name = self.name();

        if let Some(ref ty_generic) = self.ty_generic() {
            quote! { #name(#ty_generic) }
        } else {
            quote! { #name }
        }
    }

    fn ctor(&self, enum_name: &Ident, variants: &[Self]) -> TokenStream {
        let name = self.name();
        let ctor = Ident::new_raw(&name.unraw().to_string().to_snake_case(), name.span());

        let ty_generics = variants.iter().filter_map(|var| {
            if ptr::eq(var, self) {
                self.ty_generic().map(ToTokens::to_token_stream)
            } else {
                var.ty_generic()
                    .map(|_| quote! { ::std::convert::Infallible })
            }
        });

        if let Some(ty_generic) = self.ty_generic() {
            quote! {
                #[automatically_derived]
                impl<#ty_generic> #enum_name<#(#ty_generics),*> {
                    pub fn #ctor(#ctor: #ty_generic) -> Self {
                        Self::#name(#ctor)
                    }
                }
            }
        } else {
            quote! {
                #[automatically_derived]
                impl #enum_name<#(#ty_generics),*> {
                    pub fn #ctor() -> Self {
                        Self::#name
                    }
                }
            }
        }
    }
}
