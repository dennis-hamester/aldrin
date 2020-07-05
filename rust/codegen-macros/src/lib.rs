#![deny(intra_doc_link_resolution_failure)]

extern crate proc_macro;

use aldrin_codegen::{Generator, Options, RustOptions};
use aldrin_parser::diag::Line;
use aldrin_parser::{Diagnostic, Parsed, Parser};
use proc_macro::TokenStream;
use proc_macro_error::{
    abort_call_site, emit_call_site_error, emit_call_site_warning, proc_macro_error,
};
use std::env;
use std::fmt::Write;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Error, Ident, LitBool, LitStr, Result, Token};

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
        let rust_options = RustOptions::new();
        let output = match gen.generate_rust(&rust_options) {
            Ok(output) => output,
            Err(e) => panic!("{}", e),
        };

        write!(
            &mut modules,
            "pub mod {} {{ {} const _: &[u8] = include_bytes!(\"{}\"); }}",
            output.module_name,
            output.module_content,
            schema.display()
        )
        .unwrap();
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
            } else {
                return Err(Error::new_spanned(opt, "invalid option"));
            }

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(args)
    }
}

fn format_diagnostic<D>(d: &D, parsed: &Parsed) -> String
where
    D: Diagnostic,
{
    let formatted = d.format(parsed);
    let mut lines = formatted.lines.into_iter();

    let intro = match lines.next().expect("diagnostic without lines") {
        Line::Intro(intro) => intro,
        _ => panic!("first line is not Line::Intro"),
    };

    let mut msg = format!("{}\n", intro.reason);
    for line in lines {
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
