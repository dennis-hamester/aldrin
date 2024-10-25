//! # Aldrin macros
//!
//! The macros in this crate are not generally meant to be used directly, but through re-exports in
//! other crates.
//!
//! ## Procedural macros
//!
//! - [`generate`](generate!): Re-exported in crate `aldrin`
//!
//! ## Derive macros
//!
//! - [`Serialize`]
//! - [`Deserialize`]
//! - [`Introspectable`]
//! - [`SerializeKey`]
//! - [`DeserializeKey`]
//! - [`KeyTypeOf`]
//!
//! All derive macros are re-exported in both `aldrin` and `aldrin-core`.
//!
//! Note that the derive macros come in 2 variants, depending on where they are re-exported. E.g.
//! [`Serialize`] is re-exported in `aldrin-core`, but [`SerializeFromAldrin`] is re-exported in
//! `aldrin` as `Serialize`. This was done because derive macros, when used from `aldrin`, require
//! slightly different behavior.
//!
//! ### Attributes
//!
//! All derive macros support various attributes and some apply to multiple macros.
//!
//! #### Container attributes
//!
//! ##### `crate`
//!
//! - Applies to: all derive macros
//!
//! The attribute `#[aldrin(crate = "...")` can be used to override the name of the `aldrin_core`
//! crate. This is useful when `aldrin_core` is not a direct dependency, but only reexported
//! somewhere. The default value depends on from where the macro is invoked, it's either
//! `::aldrin::core` or `::aldrin_core`.
//!
//! ```
//! mod my_reexports {
//!     pub use aldrin_core as my_aldrin_core;
//! }
//!
//! #[derive(
//!     my_reexports::my_aldrin_core::Serialize,
//!     my_reexports::my_aldrin_core::Deserialize,
//! )]
//! #[aldrin(crate = "my_reexports::my_aldrin_core")]
//! struct Person {
//!     name: String,
//! }
//! ```
//!
//! ##### `{ser,de,intro,ser_key,de_key,key_ty}_bounds`
//!
//! Applies to:
//! - `ser_bounds`: `Serialize`
//! - `de_bounds`: `Deserialize`
//! - `intro_bounds`: `Introspectable`
//! - `ser_key_bounds`: `SerializeKey`
//! - `de_key_bounds`: `DeserializeKey`
//! - `key_ty_bounds`: `KeyTypeOf`
//!
//! These attributes specify the generic bounds added to `where` clauses The default is to add `T:
//! Trait` bounds for each type parameter `T` and the respective trait.
//!
//! The values of these attributes must be a string of comma-separated bounds, just like they would
//! appear in a `where` clause.
//!
//! ```
//! # use aldrin_core::{Deserialize, Serialize};
//! #[derive(Serialize, Deserialize)]
//! #[aldrin(ser_bounds = "T: aldrin::core::Serialize")]
//! #[aldrin(de_bounds = "T: aldrin::core::Deserialize")]
//! struct Person<T> {
//!     pets: Vec<T>,
//! }
//! ```
//!
//! ##### `schema`
//!
//! - Applies to: `Introspectable`
//!
//! Deriving `Introspectable` requires specifying a schema name. It is an error if this attribute is
//! missing.
//!
//! ```
//! # use aldrin_core::Introspectable;
//! #[derive(Introspectable)]
//! #[aldrin(schema = "contacts")]
//! struct Person {
//!     name: String,
//! }
//! ```
//!
//! #### Field and variant attributes
//!
//! ##### `id`
//!
//! - Applies to: `Serialize`, `Deserialize` and `Introspectable`
//!
//! Use `#[aldrin(id = ...)]` to override the automatically defined id for a field or variant.
//!
//! Default ids start at 0 for the first field or variant and then increment by 1 for each
//! subsequent field or variant.
//!
//! ```
//! # use aldrin_core::{Deserialize, Introspectable, Serialize};
//! #[derive(Serialize, Deserialize, Introspectable)]
//! #[aldrin(schema = "family_tree")]
//! struct Person {
//!     age: u8, // id = 0
//!
//!     #[aldrin(id = 5)]
//!     name: String, // id = 5
//!
//!     siblings: Vec<Self>, // id = 6
//! }
//! ```
//!
//! ```
//! # use aldrin_core::{Deserialize, Introspectable, Serialize};
//! #[derive(Serialize, Deserialize, Introspectable)]
//! #[aldrin(schema = "pets")]
//! enum Pet {
//!     Dog, // id = 0
//!
//!     #[aldrin(id = 5)]
//!     Cat, // id = 5
//!
//!     Alpaca, // id = 6
//! }
//! ```
//!
//! ##### `optional`
//!
//! - Applies to: `Serialize`, `Deserialize` and `Introspectable`
//!
//! Use `#[aldrin(optional)]` to mark fields of a struct as optional. They must be of an `Option<T>`
//! type.
//!
//! Optional fields are not serialized if `None` and are allowed to be missing when deserializing a
//! value.
//!
//! ```
//! # use aldrin_core::{Deserialize, Serialize};
//! #[derive(Serialize, Deserialize)]
//! struct MyStruct {
//!     required_field_1: i32,
//!     required_field_2: Option<i32>,
//!
//!     #[aldrin(optional)]
//!     optional_field: Option<i32>,
//! }
//! ```
//!
//! Both fields `required_field_1` and `required_field_2` will always be serialized and
//! deserialization will fail if either is missing. Serialization of `optional_field` is skipped if
//! it is `None`. If it's missing during deserialization, then it will be set to `None`.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

extern crate proc_macro;

mod codegen;
mod derive;

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

/// Derive macro for the `Serialize` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(Serialize, attributes(aldrin))]
pub fn serialize_from_core(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_serialize_from_core(input)
}

/// Derive macro for the `Serialize` trait.
///
/// This is the same as [`Serialize`], except that the `crate` defaults to `::aldrin::core`.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(SerializeFromAldrin, attributes(aldrin))]
pub fn serialize_from_aldrin(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_serialize_from_aldrin(input)
}

/// Derive macro for the `Deserialize` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(Deserialize, attributes(aldrin))]
pub fn deserialize_from_core(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_deserialize_from_core(input)
}

/// Derive macro for the `Deserialize` trait.
///
/// This is the same as [`Deserialize`], except that the `crate` defaults to `::aldrin::core`.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(DeserializeFromAldrin, attributes(aldrin))]
pub fn deserialize_from_aldrin(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_deserialize_from_aldrin(input)
}

/// Derive macro for the `Introspectable` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(Introspectable, attributes(aldrin))]
pub fn introspectable_from_core(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_introspectable_from_core(input)
}

/// Derive macro for the `Introspectable` trait.
///
/// This is the same as [`Introspectable`], except that the `crate` defaults to `::aldrin::core`.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(IntrospectableFromAldrin, attributes(aldrin))]
pub fn introspectable_from_aldrin(
    input: syn::DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_introspectable_from_aldrin(input)
}

/// Derive macro for the `SerializeKey` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(SerializeKey, attributes(aldrin))]
pub fn serialize_key_from_core(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_serialize_key_from_core(input)
}

/// Derive macro for the `SerializeKey` trait.
///
/// This is the same as [`SerializeKey`], except that the `crate` defaults to `::aldrin::core`.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(SerializeKeyFromAldrin, attributes(aldrin))]
pub fn serialize_key_from_aldrin(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_serialize_key_from_aldrin(input)
}

/// Derive macro for the `DeserializeKey` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(DeserializeKey, attributes(aldrin))]
pub fn deserialize_key_from_core(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_deserialize_key_from_core(input)
}

/// Derive macro for the `DeserializeKey` trait.
///
/// This is the same as [`DeserializeKey`], except that the `crate` defaults to `::aldrin::core`.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(DeserializeKeyFromAldrin, attributes(aldrin))]
pub fn deserialize_key_from_aldrin(
    input: syn::DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_deserialize_key_from_aldrin(input)
}

/// Derive macro for the `KeyTypeOf` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(KeyTypeOf, attributes(aldrin))]
pub fn key_type_of_from_core(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_key_type_of_from_core(input)
}

/// Derive macro for the `KeyTypeOf` trait.
///
/// This is the same as [`KeyTypeOf`], except that the `crate` defaults to `::aldrin::core`.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
#[manyhow::manyhow]
#[proc_macro_derive(KeyTypeOfFromAldrin, attributes(aldrin))]
pub fn key_type_of_from_aldrin(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    derive::gen_key_type_of_from_aldrin(input)
}
