use aldrin_codegen::{Generator, Options, RustOptions};
use aldrin_parser::{Diagnostic, Parsed, Parser};
use manyhow::{emit, Emitter};
use proc_macro2::Span;
use std::env;
use std::fmt::Write;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Ident, LitBool, LitStr, Result, Token};

pub fn generate(args: Args, emitter: &mut Emitter) -> manyhow::Result {
    let mut parser = Parser::new();
    for include in args.includes {
        parser.add_schema_path(include);
    }

    let mut modules = String::new();

    for schema in args.schemas {
        let parsed = parser.parse(&schema);

        for error in parsed.errors() {
            emit!(emitter, "{}", format_diagnostic(error, &parsed));
        }

        if args.warnings_as_errors {
            for warning in parsed.warnings() {
                emit!(emitter, "{}", format_diagnostic(warning, &parsed));
            }
        }

        if !parsed.errors().is_empty() {
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
        rust_options.introspection_if = args.introspection_if.as_deref();

        let output = match gen.generate_rust(&rust_options) {
            Ok(output) => output,

            Err(e) => {
                emit!(emitter, "Aldrin code generation failed: {e}");
                continue;
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

    emitter.into_result()?;

    modules
        .parse()
        .map_err(Error::from)
        .map_err(manyhow::Error::from)
}

pub struct Args {
    schemas: Vec<PathBuf>,
    includes: Vec<PathBuf>,
    options: Options,
    warnings_as_errors: bool,
    patches: Vec<PathBuf>,
    struct_builders: bool,
    struct_non_exhaustive: bool,
    enum_non_exhaustive: bool,
    event_non_exhaustive: bool,
    function_non_exhaustive: bool,
    introspection_if: Option<String>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let first_schema = lit_str_to_path(&input.parse::<LitStr>()?)?;

        let mut args = Args {
            schemas: vec![first_schema],
            includes: Vec::new(),
            options: Options::default(),
            warnings_as_errors: false,
            patches: Vec::new(),
            struct_builders: true,
            struct_non_exhaustive: true,
            enum_non_exhaustive: true,
            event_non_exhaustive: true,
            function_non_exhaustive: true,
            introspection_if: None,
        };

        // Additional schemas
        while !input.is_empty() {
            input.parse::<Token![,]>()?;

            let Ok(lit_str) = input.parse::<LitStr>() else {
                break;
            };

            args.schemas.push(lit_str_to_path(&lit_str)?);
        }

        // Options
        while !input.is_empty() {
            let opt: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if opt == "include" {
                let lit_str = input.parse::<LitStr>()?;
                args.includes.push(lit_str_to_path(&lit_str)?);
            } else if opt == "client" {
                args.options.client = input.parse::<LitBool>()?.value;
            } else if opt == "server" {
                args.options.server = input.parse::<LitBool>()?.value;
            } else if opt == "warnings_as_errors" {
                args.warnings_as_errors = input.parse::<LitBool>()?.value;
            } else if opt == "patch" {
                let lit_str = input.parse::<LitStr>()?;
                args.patches.push(lit_str_to_path(&lit_str)?);
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
            } else if opt == "introspection" {
                args.options.introspection = input.parse::<LitBool>()?.value;
            } else if opt == "introspection_if" {
                let lit_str = input.parse::<LitStr>()?;
                args.introspection_if = Some(lit_str.value());
                args.options.introspection = true;
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

fn format_diagnostic(diag: &impl Diagnostic, parsed: &Parsed) -> String {
    let formatted = diag.format(parsed);

    let mut msg = format!("{}\n", formatted.summary());
    for line in formatted.lines().skip(1) {
        msg.push_str(&line.to_string());
    }

    msg
}

fn lit_str_to_path(lit_str: &LitStr) -> Result<PathBuf> {
    let path = PathBuf::from(lit_str.value());

    if path.is_absolute() {
        Ok(path)
    } else {
        let mut absolute = env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .map_err(|e| {
                Error::new(
                    lit_str.span(),
                    format!("relative paths require CARGO_MANIFEST_DIR environment variable: {e}"),
                )
            })?;

        absolute.push(path);
        Ok(absolute)
    }
}
