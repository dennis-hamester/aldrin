//! # Aldrin macros
//!
//! The macros in this crate are not generally meant to be used directly, but through re-exports in
//! other crates.
//!
//! ## Procedural macros
//!
//! - [`generate`](generate!): Re-exported in crate `aldrin`
//! - [`service`](service!): Re-exported in crate `aldrin`
//!
//! ## Derive macros
//!
//! - [`Tag`]
//! - [`PrimaryTag`]
//! - [`RefType`]
//! - [`Serialize`]
//! - [`Deserialize`]
//! - [`Introspectable`]
//!
//! All derive macros are re-exported in both `aldrin` and `aldrin-core`.
//!
//! ### Attributes
//!
//! All derive macros support various attributes and some apply to multiple macros.
//!
//! #### Container attributes
//!
//! ##### `crate`
//!
//! - Applies to: [all derive macros](crate#derive-macros)
//!
//! The attribute `#[aldrin(crate = "...")` can be used to override the path of the `aldrin_core`
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
//!     my_reexports::my_aldrin_core::Tag,
//!     my_reexports::my_aldrin_core::PrimaryTag,
//!     my_reexports::my_aldrin_core::RefType,
//!     my_reexports::my_aldrin_core::Serialize,
//!     my_reexports::my_aldrin_core::Deserialize,
//! )]
//! #[aldrin(crate = "my_reexports::my_aldrin_core")]
//! struct Person {
//!     name: String,
//! }
//! ```
//!
//! ##### `schema`
//!
//! - Applies to: [`Introspectable`]
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
//! ##### `ref_type`
//!
//! - Applies to: [`RefType`], [`Serialize`]
//!
//! Controls the name of the ref type. If the attribute is omitted, then a default is constructed by
//! appending `Ref` to the type's name.
//!
//! The `Serialize` derive macro assume that a ref type exists. If this is not desired, then use
//! `ref_type = ""` to disable that.
//!
//! #### Field and variant attributes
//!
//! ##### `id`
//!
//! - Applies to: [`Serialize`], [`Deserialize`] and [`Introspectable`]
//!
//! Use `#[aldrin(id = ...)]` to override the automatically defined id for a field or variant.
//!
//! Default ids start at 0 for the first field or variant and then increment by 1 for each
//! subsequent field or variant.
//!
//! ```
//! # use aldrin_core::{Deserialize, Introspectable, PrimaryTag, RefType, Serialize, Tag};
//! #[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
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
//! # use aldrin_core::{Deserialize, Introspectable, PrimaryTag, RefType, Serialize, Tag};
//! #[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
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
//! - Applies to: [`Serialize`], [`Deserialize`] and [`Introspectable`]
//!
//! Use `#[aldrin(optional)]` to mark fields of a struct as optional. They must be of an `Option<T>`
//! type.
//!
//! Optional fields are not serialized if `None` and are allowed to be missing when deserializing a
//! value.
//!
//! ```
//! # use aldrin_core::{Deserialize, Introspectable, PrimaryTag, RefType, Serialize, Tag};
//! #[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
//! #[aldrin(schema = "example")]
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
//!
//! ##### `fallback`
//!
//! - Applies to: [`Serialize`], [`Deserialize`] and [`Introspectable`]
//!
//! The last field of a struct and the last variant of an enum can optionally be marked with
//! `#[aldrin(fallback)]`. This will enable successful serialization and deserialization of unknown
//! fields and variants. For structs, the field type must be `aldrin_core::UnknownFields`. For
//! enums, the variant must have a single field of type `aldrin_core::UnknownVariant`.
//!
//! This attribute cannot be combined with `#[aldrin(optional)]`.
//!
//! Example of a struct with a fallback field:
//! ```
//! # use aldrin_core::{Deserialize, Introspectable, PrimaryTag, RefType, Serialize, Tag, UnknownFields};
//! #[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
//! #[aldrin(schema = "contacts")]
//! struct Person {
//!     name: String,
//!     age: u8,
//!
//!     #[aldrin(fallback)]
//!     unknown_fields: UnknownFields,
//! }
//! ```
//!
//! Example of an enum with a fallback variant:
//! ```
//! # use aldrin_core::{Deserialize, Introspectable, PrimaryTag, RefType, Serialize, Tag, UnknownVariant};
//! #[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize, Introspectable)]
//! #[aldrin(schema = "zoo")]
//! enum AnimalType {
//!     Alpaca,
//!     Pig,
//!
//!     #[aldrin(fallback)]
//!     Unkown(UnknownVariant),
//! }
//! ```

#![deny(missing_docs)]

extern crate proc_macro;

mod codegen;
mod derive;
mod service;
#[cfg(test)]
mod test;

use proc_macro2::TokenStream;
use syn::{DeriveInput, Result};

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
///     example1::MyStruct {
///         field1: Some(1),
///         field2: None,
///     };
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
///     schema::MyStruct { // Note `schema` instead of `example1`.
///         field1: Some(1),
///         field2: None,
///     };
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
///     example3::Foo {
///         bar: Some(example4::Bar {
///             baz: Some(12),
///         }),
///     };
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
///     example1::MyStructRenamed {
///         field1: Some(1),
///         field2: None,
///     };
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
///     example1::MyStructRenamedAgain {
///         field1: Some(1),
///         field2: None,
///     };
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
///
/// # Overriding the path of the `aldrin` crate
///
/// The macro assumes per default that the `aldrin` crate is available as `::aldrin`. This can be
/// overridden with the `krate` attribute. Note that [`generate!`] creates a new module and that
/// path resolution starts inside that module.
///
/// ```
/// # use aldrin_macros::generate;
/// # fn main() {}
/// mod my_reexports {
///     pub use aldrin as my_aldrin;
/// }
///
/// generate! {
///     "schemas/example1.aldrin",
///     crate = "super::my_reexports::my_aldrin",
/// }
/// ```
#[manyhow::manyhow]
#[proc_macro]
pub fn generate(args: codegen::Args, emitter: &mut manyhow::Emitter) -> manyhow::Result {
    codegen::generate(args, emitter)
}

/// Defines a service and proxy type.
///
/// The form this macro takes closely resembles that of services in Aldrin schema, but it uses
/// actual Rust expressions and types.
///
/// ```
/// # use aldrin::core::ServiceUuid;
/// # use aldrin_macros::{service, Deserialize, PrimaryTag, RefType, Serialize, Tag};
/// # use uuid::uuid;
/// service! {
///     pub service Echo {
///         uuid = ServiceUuid(uuid!("ee98534d-345a-4399-a656-07fd9c39a96e"));
///         version = 1;
///
///         fn echo @ 1 {
///             args = String;
///             ok = String;
///             err = Error;
///         }
///
///         fn echo_all @ 2 {
///             args = String;
///             err = Error;
///         }
///
///         event echoed_to_all @ 1 = String;
///     }
/// }
///
/// #[derive(Tag, PrimaryTag, RefType, Serialize, Deserialize)]
/// pub enum Error {
///     EmptyString,
/// }
/// ```
///
/// # Overriding the path to the `aldrin` crate
///
/// Use the `#[aldrin(crate = "...")]` attribute to override the path to the `aldrin` crate.
///
/// ```
/// # use aldrin::core::ServiceUuid;
/// # use aldrin_macros::service;
/// # use uuid::uuid;
/// mod my_reexports {
///     pub use aldrin as my_aldrin;
/// }
///
/// service! {
///     #[aldrin(crate = "my_reexports::my_aldrin")]
///     pub service Ping {
///         uuid = ServiceUuid(uuid!("b6633b9f-c26d-4987-8ec0-5c8e526290f9"));
///         version = 1;
///     }
/// }
/// ```
///
/// # Generating only client or server code
///
/// Client and server code generation can be disabled individually with the `#[aldrin(no_client)]`
/// and `#[aldrin(no_server)]` attributes.
///
/// The following examples uses `no_client` and thus no `PingProxy` type will be generated.
///
/// ```
/// # use aldrin::core::ServiceUuid;
/// # use aldrin_macros::service;
/// # use uuid::uuid;
/// service! {
///     #[aldrin(no_client)]
///     pub service Ping {
///         uuid = ServiceUuid(uuid!("b6633b9f-c26d-4987-8ec0-5c8e526290f9"));
///         version = 1;
///     }
/// }
/// ```
///
/// # Introspection
///
/// The `Introspectable` trait can be implemented automatically by specifying the
/// `#[aldrin(introspection)]` attribute. It also requires the `#[aldrin(schema = "...")]` attribute
/// and all referenced types must be `Introspectable`.
///
/// ```
/// # use aldrin::core::ServiceUuid;
/// # use aldrin_macros::service;
/// # use uuid::uuid;
/// service! {
///     #[aldrin(schema = "ping", introspection)]
///     pub service Ping {
///         uuid = ServiceUuid(uuid!("b6633b9f-c26d-4987-8ec0-5c8e526290f9"));
///         version = 1;
///     }
/// }
/// ```
///
/// It is also possible to implement `Introspectable` conditionally depending on some Cargo feature
/// with the `#[aldrin(introspection_if = "...")]` attribute.
///
/// ```
/// # use aldrin::core::ServiceUuid;
/// # use aldrin_macros::service;
/// # use uuid::uuid;
/// service! {
///     #[aldrin(schema = "ping", introspection_if = "introspection")]
///     pub service Ping {
///         uuid = ServiceUuid(uuid!("b6633b9f-c26d-4987-8ec0-5c8e526290f9"));
///         version = 1;
///     }
/// }
/// ```
#[manyhow::manyhow]
#[proc_macro]
pub fn service(svc: service::Service) -> TokenStream {
    svc.generate()
}

/// Derive macro for the `Tag` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
#[manyhow::manyhow]
#[proc_macro_derive(Tag, attributes(aldrin))]
pub fn tag_from_core(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_tag_from_core(input)
}

/// Derive macro for the `Tag` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
#[doc(hidden)]
#[manyhow::manyhow]
#[proc_macro_derive(TagFromAldrin, attributes(aldrin))]
pub fn tag_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_tag_from_aldrin(input)
}

/// Derive macro for the `PrimaryTag` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
#[manyhow::manyhow]
#[proc_macro_derive(PrimaryTag, attributes(aldrin))]
pub fn primary_tag_from_core(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_primary_tag_from_core(input)
}

/// Derive macro for the `PrimaryTag` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
#[doc(hidden)]
#[manyhow::manyhow]
#[proc_macro_derive(PrimaryTagFromAldrin, attributes(aldrin))]
pub fn primary_tag_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_primary_tag_from_aldrin(input)
}

/// Derive macro for ref types.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
/// - [`ref_type`](crate#ref_type)
#[manyhow::manyhow]
#[proc_macro_derive(RefType, attributes(aldrin))]
pub fn ref_type_from_core(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_ref_type_from_core(input)
}

/// Derive macro for ref types.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
/// - [`ref_type`](crate#ref_type)
#[doc(hidden)]
#[manyhow::manyhow]
#[proc_macro_derive(RefTypeFromAldrin, attributes(aldrin))]
pub fn ref_type_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_ref_type_from_aldrin(input)
}

/// Derive macro for the `Serialize` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
/// - [`ref_type`](crate#ref_type)
/// - [`id`](crate#id)
/// - [`optional`](crate#optional)
/// - [`fallback`](crate#fallback)
#[manyhow::manyhow]
#[proc_macro_derive(Serialize, attributes(aldrin))]
pub fn serialize_from_core(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_serialize_from_core(input)
}

/// Derive macro for the `Serialize` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
/// - [`ref_type`](crate#ref_type)
/// - [`id`](crate#id)
/// - [`optional`](crate#optional)
/// - [`fallback`](crate#fallback)
#[doc(hidden)]
#[manyhow::manyhow]
#[proc_macro_derive(SerializeFromAldrin, attributes(aldrin))]
pub fn serialize_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_serialize_from_aldrin(input)
}

/// Derive macro for the `Deserialize` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
/// - [`id`](crate#id)
/// - [`optional`](crate#optional)
/// - [`fallback`](crate#fallback)
#[manyhow::manyhow]
#[proc_macro_derive(Deserialize, attributes(aldrin))]
pub fn deserialize_from_core(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_deserialize_from_core(input)
}

/// Derive macro for the `Deserialize` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
/// - [`id`](crate#id)
/// - [`optional`](crate#optional)
/// - [`fallback`](crate#fallback)
#[doc(hidden)]
#[manyhow::manyhow]
#[proc_macro_derive(DeserializeFromAldrin, attributes(aldrin))]
pub fn deserialize_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_deserialize_from_aldrin(input)
}

/// Derive macro for the `Introspectable` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
/// - [`schema`](crate#schema)
/// - [`id`](crate#id)
/// - [`optional`](crate#optional)
/// - [`fallback`](crate#fallback)
#[manyhow::manyhow]
#[proc_macro_derive(Introspectable, attributes(aldrin))]
pub fn introspectable_from_core(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_introspectable_from_core(input)
}

/// Derive macro for the `Introspectable` trait.
///
/// See the [crate-level](crate#attributes) documentation in the `aldrin-macros` crate for more
/// information about the supported attributes.
///
/// Relevant attributes:
/// - [`crate`](crate#crate)
/// - [`schema`](crate#schema)
/// - [`id`](crate#id)
/// - [`optional`](crate#optional)
/// - [`fallback`](crate#fallback)
#[doc(hidden)]
#[manyhow::manyhow]
#[proc_macro_derive(IntrospectableFromAldrin, attributes(aldrin))]
pub fn introspectable_from_aldrin(input: DeriveInput) -> Result<TokenStream> {
    derive::gen_introspectable_from_aldrin(input)
}
