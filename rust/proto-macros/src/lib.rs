mod de;
mod options;
mod ser;

use proc_macro::TokenStream;
use syn::{DeriveInput, Error};

/// Derive macro for the Serialize trait
///
/// Note: this documentation also applies to the [`Deserialize`] derive macro.
///
/// # Container attributes
///
/// ## `crate`
///
/// The attribute `#[aldrin(crate = "...")` can be used to override the name of the `aldrin_proto`
/// crate. This is useful when `aldrin_proto` is not a direct dependency, but only reexported
/// somewhere.
///
/// ```
/// mod my_reexports {
///     pub use aldrin_proto as my_aldrin_proto;
/// }
///
/// #[derive(
///     my_reexports::my_aldrin_proto::Serialize,
///     my_reexports::my_aldrin_proto::Deserialize,
/// )]
/// #[aldrin(crate = "my_reexports::my_aldrin_proto")]
/// struct MyStruct {
///     field1: u32,
/// }
/// ```
///
/// # Field and variant attributes
///
/// ## `id`
///
/// Use `#[aldrin(id = ...)]` to override the automatically defined id for a field or variant.
///
/// Default ids start at 0 for the first field or variant and then increment by 1 for each
/// subsequent field or variant.
///
/// ```
/// #[derive(aldrin_proto::Serialize, aldrin_proto::Deserialize)]
/// struct MyStruct {
///     field1: u32, // id = 0
///
///     #[aldrin(id = 5)]
///     field2: u32, // id = 5
///
///     field3: u32, // id = 6
/// }
/// ```
///
/// ```
/// #[derive(aldrin_proto::Serialize, aldrin_proto::Deserialize)]
/// enum MyEnum {
///     Variant1, // id = 0
///
///     #[aldrin(id = 5)]
///     Variant2, // id = 5
///
///     Variant3, // id = 6
/// }
/// ```
#[proc_macro_derive(Serialize, attributes(aldrin))]
pub fn serialize(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    ser::gen_serialize(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derive macro for the Deserialize trait
///
/// See the documentation of the [`Serialize`] derive macro, which also applies for this macro.
#[proc_macro_derive(Deserialize, attributes(aldrin))]
pub fn deserialize(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    de::gen_deserialize(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
