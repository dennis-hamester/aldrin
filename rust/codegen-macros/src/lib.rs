//! Aldrin code generation macros
//!
//! This crate provides a single macro as an alternative frontend to the Aldrin code generator. It
//! removes the need to generate code from a schema beforehand or as part of a `build.rs`.
//!
//! See the documentation of the [`generate!`] macro for more information and usage examples.

#![allow(clippy::needless_doctest_main)]
#![deny(broken_intra_doc_links)]
#![deny(missing_docs)]

extern crate proc_macro;

use aldrin_codegen::{Generator, Options, RustOptions};
use aldrin_parser::{Diagnostic, Parsed, Parser};
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{
    abort_call_site, emit_call_site_error, emit_call_site_warning, proc_macro_error,
};
use std::env;
use std::fmt::Write;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Error, Ident, LitBool, LitStr, Result, Token};

/// Generates code from an Aldrin schema.
///
/// # Basic usage
///
/// The [`generate!`] macro takes one required argument, the path to the schema file. Paths can be
/// relative to `Cargo.toml` file. This requires building with Cargo (or more specifically, the
/// `CARGO_MANIFEST_DIR` environment variable). Building without Cargo currently supports only
/// absolute paths.
///
/// The generated code depends only the `aldrin-client` crate. Make sure you have it specified as a
/// dependency in your `Cargo.toml`.
///
/// ```
/// aldrin_codegen_macros::generate!("schemas/example1.aldrin");
///
/// fn main() {
///     example1::MyStruct::builder()
///         .set_field1(Some(12))
///         .set_field2(Some(34))
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
///     aldrin_codegen_macros::generate!("schemas/example1.aldrin");
/// }
/// ```
///
/// If you have only a single schema, it is occasionally convenient to put the generated code inside
/// another module (like above), but then also re-export everything into it:
///
/// ```
/// mod schema {
///     aldrin_codegen_macros::generate!("schemas/example1.aldrin");
///     pub use example1::*;
/// }
///
/// fn main() {
///     schema::MyStruct::builder() // Note `schema` instead of `example1`.
///         .set_field1(Some(12))
///         .set_field2(Some(34))
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
/// aldrin_codegen_macros::generate! {
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
/// aldrin_codegen_macros::generate! {
///     "schemas/example3.aldrin",
///     "schemas/example4.aldrin",
///     include = "schemas",
/// }
///
/// fn main() {
///     example3::Foo::builder()
///         .set_bar(Some(example4::Bar::builder().set_baz(Some(12)).build()))
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
/// aldrin_codegen_macros::generate! {
///     "schemas/example1.aldrin",
///     patch = "schemas/example1-rename.patch",
/// }
///
/// fn main() {
///     example1::MyStructRenamed::builder()
///         .set_field1(Some(12))
///         .set_field2(Some(34))
///         .build();
/// }
/// ```
///
/// Patches are applied in the order they are specified.
///
/// ```
/// aldrin_codegen_macros::generate! {
///     "schemas/example1.aldrin",
///     patch = "schemas/example1-rename.patch",
///     patch = "schemas/example1-rename-again.patch",
/// }
///
/// fn main() {
///     example1::MyStructRenamedAgain::builder()
///         .set_field1(Some(12))
///         .set_field2(Some(34))
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
/// aldrin_codegen_macros::generate! {
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
/// aldrin_codegen_macros::generate! {
///     "schemas/example1.aldrin",
///     struct_non_exhaustive = false,
///     enum_non_exhaustive = false,
///     event_non_exhaustive = false,
///     function_non_exhaustive = false,
/// }
/// ```
///
/// # Errors and warnings
///
/// Any errors and warnings from the schemas will be shown as part of the regular compiler
/// output. No code will be generated, if there are any errors in the schemas.
///
/// Warnings can currently only be emitted on nightly Rust. They are silently ignored on stable and
/// beta. Unfortunately, this may suppress important diagnostics about your schemas. You can use the
/// option `warnings_as_errors = true` to treat all warnings as errors.
///
/// On the other hand, if you are on nightly Rust and generate code for a foreign schema, which you
/// have no direct influence on, warnings may clutter your compiler output. In that case you can use
/// `suppress_warnings = true` to ignore all warnings.
///
/// In general, it is advisable to use `warnings_as_errors` for your own schemas, and
/// `suppress_warnings` for foreign schemas:
///
/// ```
/// // Own schema
/// aldrin_codegen_macros::generate! {
///     "schemas/example5.aldrin",
///     include = "schemas/foreign",
///     warnings_as_errors = true,
/// }
///
/// // Foreign schema
/// aldrin_codegen_macros::generate! {
///     "schemas/foreign/example6.aldrin",
///     suppress_warnings = true,
/// }
/// # fn main() {}
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn generate(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);

    let mut parser = Parser::new();
    for include in args.includes {
        parser.add_schema_path(include);
    }

    let mut modules = String::new();
    let mut abort = false;

    for schema in args.schemas {
        let parsed = parser.parse(&schema);

        for error in parsed.errors() {
            let msg = format_diagnostic(error, &parsed);
            emit_call_site_error!(msg);
        }

        if !args.suppress_warnings || args.warnings_as_errors {
            for warning in parsed.warnings() {
                let msg = format_diagnostic(warning, &parsed);

                if args.warnings_as_errors {
                    emit_call_site_error!(msg);
                } else {
                    emit_call_site_warning!(msg);
                }
            }
        }

        if !parsed.errors().is_empty() {
            abort |= true;
            continue;
        }

        let gen = Generator::new(&args.options, &parsed);

        let mut rust_options = RustOptions::new();
        for patch in &args.patches {
            rust_options.patches.push(patch);
        }
        rust_options.struct_builders = args.struct_builders;
        rust_options.struct_non_exhaustive = args.struct_non_exhaustive;
        rust_options.enum_non_exhaustive = args.enum_non_exhaustive;
        rust_options.event_non_exhaustive = args.event_non_exhaustive;
        rust_options.function_non_exhaustive = args.function_non_exhaustive;

        let output = match gen.generate_rust(&rust_options) {
            Ok(output) => output,
            Err(e) => {
                emit_call_site_error!("{}", e);
                abort_call_site!("there were Aldrin schema errors");
            }
        };

        write!(
            &mut modules,
            "pub mod {} {{ {} const _: &[u8] = include_bytes!(\"{}\"); ",
            output.module_name,
            output.module_content,
            schema.display()
        )
        .unwrap();

        for patch in &args.patches {
            write!(
                &mut modules,
                "const _: &[u8] = include_bytes!(\"{}\"); ",
                patch.display()
            )
            .unwrap();
        }

        write!(&mut modules, "}}").unwrap();
    }

    if abort {
        abort_call_site!("there were Aldrin schema errors");
    }

    modules.parse().unwrap()
}

struct Args {
    schemas: Vec<PathBuf>,
    includes: Vec<PathBuf>,
    options: Options,
    warnings_as_errors: bool,
    suppress_warnings: bool,
    patches: Vec<PathBuf>,
    struct_builders: bool,
    struct_non_exhaustive: bool,
    enum_non_exhaustive: bool,
    event_non_exhaustive: bool,
    function_non_exhaustive: bool,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let first_schema = lit_str_to_path(input.parse::<LitStr>()?);
        let mut args = Args {
            schemas: vec![first_schema],
            includes: Vec::new(),
            options: Options::default(),
            warnings_as_errors: false,
            suppress_warnings: false,
            patches: Vec::new(),
            struct_builders: true,
            struct_non_exhaustive: true,
            enum_non_exhaustive: true,
            event_non_exhaustive: true,
            function_non_exhaustive: true,
        };

        // Additional schemas
        while !input.is_empty() {
            input.parse::<Token![,]>()?;

            let lit_str = match input.parse::<LitStr>() {
                Ok(lit_str) => lit_str,
                Err(_) => break,
            };

            args.schemas.push(lit_str_to_path(lit_str));
        }

        // Options
        while !input.is_empty() {
            let opt: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if opt == "include" {
                let lit_str = input.parse::<LitStr>()?;
                args.includes.push(lit_str_to_path(lit_str));
            } else if opt == "client" {
                args.options.client = input.parse::<LitBool>()?.value;
            } else if opt == "server" {
                args.options.server = input.parse::<LitBool>()?.value;
            } else if opt == "warnings_as_errors" {
                args.warnings_as_errors = input.parse::<LitBool>()?.value;
            } else if opt == "suppress_warnings" {
                args.suppress_warnings = input.parse::<LitBool>()?.value;
            } else if opt == "patch" {
                let lit_str = input.parse::<LitStr>()?;
                args.patches.push(lit_str_to_path(lit_str));
            } else if opt == "struct_builders" {
                args.struct_builders = input.parse::<LitBool>()?.value;
            } else if opt == "struct_non_exhaustive" {
                args.struct_non_exhaustive = input.parse::<LitBool>()?.value;
            } else if opt == "enum_non_exhaustive" {
                args.enum_non_exhaustive = input.parse::<LitBool>()?.value;
            } else if opt == "event_non_exhaustive" {
                args.event_non_exhaustive = input.parse::<LitBool>()?.value;
            } else if opt == "function_non_exhaustive" {
                args.function_non_exhaustive = input.parse::<LitBool>()?.value;
            } else {
                return Err(Error::new_spanned(opt, "invalid option"));
            }

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        if (args.schemas.len() > 1) && !args.patches.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "patches cannot be applied to multiple schemas",
            ));
        }

        Ok(args)
    }
}

fn format_diagnostic<D>(d: &D, parsed: &Parsed) -> String
where
    D: Diagnostic,
{
    let formatted = d.format(parsed);

    let mut msg = format!("{}\n", formatted.summary());
    for line in formatted.lines().skip(1) {
        msg.push_str(&line.to_string());
    }

    msg
}

fn lit_str_to_path(lit_str: LitStr) -> PathBuf {
    let path = PathBuf::from(lit_str.value());

    if path.is_absolute() {
        path
    } else {
        let mut absolute = PathBuf::from(
            env::var("CARGO_MANIFEST_DIR")
                .expect("relative paths require CARGO_MANIFEST_DIR environment variable"),
        );
        absolute.push(path);
        absolute
    }
}
