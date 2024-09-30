//! Aldrin macros
//!
//! The macros provided by this crate depend on the enabled Cargo features. All macros are also
//! available as re-exports in other Aldrin crates. It is generally advised to use those re-exports
//! instead of depending directly on this crate.
//!
//! # `generate!()`
//!
//! - Cargo feature: `codegen`
//! - Re-exported in crate: `aldrin`

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

extern crate proc_macro;

#[cfg(feature = "codegen")]
mod codegen;
#[cfg(feature = "derive")]
mod derive;

#[cfg(feature = "codegen")]
/// Generates code from an Aldrin schema.
///
/// This macro provides a front-end to the Aldrin code generator. It is an alternative to running
/// the standalone `aldrin-gen` tool.
///
/// # Basic usage
///
/// The [`generate!`] macro takes one required argument, the path to the schema file. Paths can be
/// relative to `Cargo.toml` file. This requires building with Cargo (or more specifically, the
/// `CARGO_MANIFEST_DIR` environment variable). Building without Cargo currently supports only
/// absolute paths.
///
/// The generated code depends only the `aldrin` crate. Make sure you have it specified as a
/// dependency in your `Cargo.toml`.
///
/// ```
/// # use aldrin_macros::generate;
/// generate!("schemas/example1.aldrin");
///
/// fn main() {
///     example1::MyStruct::builder()
///         .field1(12)
///         .field2(34)
///         .build();
/// }
/// ```
///
/// This generates the module `example1` with the same content as if the stand-alone code generator
/// was used.
///
/// The module has `pub` visibility, which is not always desired, especially in library crates. A
/// common pattern is to put the generated modules inside an additional `schemas` module:
///
/// ```
/// mod schemas {
///     # use aldrin_macros::generate;
///     generate!("schemas/example1.aldrin");
/// }
/// ```
///
/// If you have only a single schema, it is occasionally convenient to put the generated code inside
/// another module (like above), but then also re-export everything into it:
///
/// ```
/// mod schema {
///     # use aldrin_macros::generate;
///     generate!("schemas/example1.aldrin");
///     pub use example1::*;
/// }
///
/// fn main() {
///     schema::MyStruct::builder() // Note `schema` instead of `example1`.
///         .field1(12)
///         .field2(34)
///         .build();
/// }
/// ```
///
/// # Multiple schemas
///
/// It is possible to pass additional paths to the macro. Code will then be generated for all of
/// them:
///
/// ```
/// # use aldrin_macros::generate;
/// generate! {
///     "schemas/example1.aldrin",
///     "schemas/example2.aldrin",
/// }
/// # fn main() {}
/// ```
///
/// Any additional options (see below) will be applied to all schemas. If this is not desired, then
/// the macro can be called multiple times instead.
///
/// # Include directories
///
/// You can specify include directories with `include = "path"`:
///
/// ```
/// # use aldrin_macros::generate;
/// generate! {
///     "schemas/example3.aldrin",
///     "schemas/example4.aldrin",
///     include = "schemas",
/// }
///
/// fn main() {
///     example3::Foo::builder()
///         .bar(example4::Bar::builder().baz(12).build())
///         .build();
/// }
/// ```
///
/// The `include` option can be repeated multiple times.
///
/// # Skipping server or client code
///
/// You can skip generating server or client code for services by setting `server = false` or
/// `client = false`. This will only affect services and types defined inside (inline structs and
/// enums), but not other top-level definitions.
///
/// Both settings default to `true`.
///
/// # Patching the generated code
///
/// You can specify additional patch files, which will be applied to the generated code. This allows
/// for arbitrary changes, such as for example custom additional derives.
///
/// Patches can only be specified when generating code for a single schema.
///
/// ```
/// # use aldrin_macros::generate;
/// generate! {
///     "schemas/example1.aldrin",
///     patch = "schemas/example1-rename.patch",
/// }
///
/// fn main() {
///     example1::MyStructRenamed::builder()
///         .field1(12)
///         .field2(34)
///         .build();
/// }
/// ```
///
/// Patches are applied in the order they are specified.
///
/// ```
/// # use aldrin_macros::generate;
/// generate! {
///     "schemas/example1.aldrin",
///     patch = "schemas/example1-rename.patch",
///     patch = "schemas/example1-rename-again.patch",
/// }
///
/// fn main() {
///     example1::MyStructRenamedAgain::builder()
///         .field1(12)
///         .field2(34)
///         .build();
/// }
/// ```
///
/// # Omitting struct builders
///
/// For every struct in the schema, usually a corresponding builder is generated as well. This can
/// be turned off by setting `struct_builders = false`.
///
/// ```
/// # use aldrin_macros::generate;
/// generate! {
///     "schemas/example1.aldrin",
///     struct_builders = false,
/// }
///
/// fn main() {
///     // example1::MyStruct::builder() and example1::MyStructBuilder are not generated
///
///     let my_struct = example1::MyStruct {
///         field1: Some(42),
///         field2: None,
///     };
/// }
/// ```
///
/// # Omitting `#[non_exhaustive]` attribute
///
/// The `#[non_exhaustive]` attribute can optionally be skipped on structs, enums, service event
/// enums and service function enums. Set one or more of:
///
/// - `struct_non_exhaustive = false`
/// - `enum_non_exhaustive = false`
/// - `event_non_exhaustive = false`
/// - `function_non_exhaustive = false`
///
/// ```
/// # use aldrin_macros::generate;
/// generate! {
///     "schemas/example1.aldrin",
///     struct_non_exhaustive = false,
///     enum_non_exhaustive = false,
///     event_non_exhaustive = false,
///     function_non_exhaustive = false,
/// }
/// ```
///
/// # Enabling introspection
///
/// To enable introspection support, pass `introspection = true` to the macro. This additionally
/// requires enabling the `introspection` Cargo feature of the `aldrin` crate.
///
/// ```
/// # use aldrin_macros::generate;
/// generate! {
///     "schemas/example1.aldrin",
///     introspection = true,
/// }
/// ```
///
/// It is also possible to conditionally enable introspection based on some Cargo feature by setting
/// `introspection_if`. This implies setting `introspection = true`. The following example will have
/// introspection code generated, but guards of the form `#[cfg(feature = "introspection")]` added.
///
/// ```
/// # use aldrin_macros::generate;
/// generate! {
///     "schemas/example1.aldrin",
///     introspection_if = "introspection",
/// }
/// ```
///
/// # Errors and warnings
///
/// Any errors from the schemas will be shown as part of the regular compiler output and no code
/// will be generated.
///
/// Warnings are currently not emitted, due to limitations on stable Rust. Unfortunately, this may
/// suppress important diagnostics about your schemas. You can use the option
/// `warnings_as_errors = true` to treat all warnings as errors.
///
/// ```compile_fail
/// # use aldrin_macros::generate;
/// generate! {
///     "schemas/example5.aldrin",
///     warnings_as_errors = true,
/// }
/// # fn main() {}
/// ```
#[manyhow::manyhow]
#[proc_macro]
pub fn generate(args: codegen::Args, emitter: &mut manyhow::Emitter) -> manyhow::Result {
    codegen::generate(args, emitter)
}

#[cfg(feature = "derive")]
/// Derive macro for the `Serialize` trait.
///
/// Note: this documentation also applies to the [`Deserialize`] derive macro.
///
/// # Container attributes
///
/// ## `crate`
///
/// The attribute `#[aldrin(crate = "...")` can be used to override the name of the `aldrin_core`
/// crate. This is useful when `aldrin_core` is not a direct dependency, but only reexported
/// somewhere. The default is `::aldrin_core`.
///
/// Note that there are also the [`SerializeFromAldrin`] and [`DeserializeFromAldrin`] macros, for
/// which `crate` defaults to `::aldrin::core`. These macros are re-exported in `aldrin` as
/// `Serialize` and `Deserialize`.
///
/// ```
/// mod my_reexports {
///     pub use aldrin_core as my_aldrin_core;
/// }
///
/// #[derive(
///     my_reexports::my_aldrin_core::Serialize,
///     my_reexports::my_aldrin_core::Deserialize,
/// )]
/// #[aldrin(crate = "my_reexports::my_aldrin_core")]
/// struct MyStruct {
///     field1: u32,
/// }
/// ```
///
/// ## `bounds`, `ser_bounds`, `de_bounds`
///
/// Per default, additional bounds `T: Serialize` and `T: Deserialize` are added for each type
/// parameter `T`. Use `ser_bounds` and `de_bounds` to override or inhibit this.
///
/// `bounds` is a shorthand for setting `ser_bounds` and `de_bounds` simultaneously.
///
/// The attribute's value should be a string literal containing bounds as they would appear in a
/// where clause. Multiple bounds can be specified by separating them with a comma. Setting either
/// attribute to an empty string will inhibit the default bounds.
///
/// The following example defines a newtype struct for [`Cow`](std::borrow::Cow). This fails to
/// compile, because `Cow`'s `Deserialize` implementation requires `T::Owned: Deserialize`. The
/// default bound `T: Deserialize` is wrong in this case.
///
/// ```compile_fail
/// # use aldrin_core::{Deserialize, Serialize};
/// # use std::borrow::Cow;
/// #[derive(Serialize, Deserialize)]
/// struct MyStruct<'a, T>(Cow<'a, T>)
/// where
///     T: ToOwned + ?Sized + 'a;
/// ```
///
/// ```text
/// error[E0277]: the trait bound `<T as ToOwned>::Owned: Deserialize` is not satisfied
///    --> src/lib.rs
///     |
/// 1   | #[derive(Serialize, Deserialize)]
///     |                     ^^^^^^^^^^^ the trait `Deserialize` is not implemented for `<T as ToOwned>::Owned`
///     |
///     = note: required for `Cow<'_, T>` to implement `Deserialize`
/// note: required by a bound in `FieldDeserializer::<'a, 'b>::deserialize`
/// ```
///
/// The solution is to override the default bounds with `de_bounds`:
///
/// ```
/// # use aldrin_core::{Deserialize, Serialize};
/// # use std::borrow::Cow;
/// #[derive(Serialize, Deserialize)]
/// #[aldrin(de_bounds = "T::Owned: aldrin_core::Deserialize")]
/// struct MyStruct<'a, T>(Cow<'a, T>)
/// where
///     T: ToOwned + ?Sized + 'a;
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
/// # use aldrin_core::{Deserialize, Serialize};
/// #[derive(Serialize, Deserialize)]
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
/// # use aldrin_core::{Deserialize, Serialize};
/// #[derive(Serialize, Deserialize)]
/// enum MyEnum {
///     Variant1, // id = 0
///
///     #[aldrin(id = 5)]
///     Variant2, // id = 5
///
///     Variant3, // id = 6
/// }
/// ```
///
/// ## `optional`
///
/// Use `#[aldrin(optional)]` to mark fields of a struct as optional. They must be of an `Option<T>`
/// type.
///
/// Optional fields are not serialized if `None` and are allowed to be missing in the serialized
/// form.
///
/// ```
/// # use aldrin_core::{Deserialize, Serialize};
/// #[derive(Serialize, Deserialize)]
/// struct MyStruct {
///     required_field_1: i32,
///     required_field_2: Option<i32>,
///
///     #[aldrin(optional)]
///     optional_field: Option<i32>,
/// }
/// ```
///
/// Both fields `required_field_1` and `required_field_2` will always be serialized and
/// deserialization will fail if they are missing. Serialization of `optional_field` is skipped if
/// it is `None`. If it's missing during deserialization, then it will be set to `None`.
#[manyhow::manyhow]
#[proc_macro_derive(Serialize, attributes(aldrin))]
pub fn serialize_from_core(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_serialize_from_core(input)
}

#[cfg(feature = "derive")]
/// Derive macro for the `Serialize` trait.
///
/// See the documentation of the [`Serialize`] derive macro in `aldrin_macros` for more information.
///
/// This is the same as [`Serialize`], except that the `crate` defaults to `::aldrin::core`.
#[manyhow::manyhow]
#[proc_macro_derive(SerializeFromAldrin, attributes(aldrin))]
pub fn serialize_from_aldrin(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_serialize_from_aldrin(input)
}

#[cfg(feature = "derive")]
/// Derive macro for the `Deserialize` trait.
///
/// See the documentation of the [`Serialize`] derive macro, which also applies for this macro.
#[manyhow::manyhow]
#[proc_macro_derive(Deserialize, attributes(aldrin))]
pub fn deserialize_from_core(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_deserialize_from_core(input)
}

#[cfg(feature = "derive")]
/// Derive macro for the `Deserialize` trait.
///
/// See the documentation of the [`Serialize`] derive macro in `aldrin_macros` for more information.
///
/// This is the same as [`Deserialize`], except that the `crate` defaults to `::aldrin::core`.
#[manyhow::manyhow]
#[proc_macro_derive(DeserializeFromAldrin, attributes(aldrin))]
pub fn deserialize_from_aldrin(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_deserialize_from_aldrin(input)
}
