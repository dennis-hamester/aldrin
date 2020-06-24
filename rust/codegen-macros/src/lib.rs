#![deny(intra_doc_link_resolution_failure)]

extern crate proc_macro;

use aldrin_codegen::{Generator, Options, RustOptions};
use aldrin_parser::diag::Line;
use aldrin_parser::{Diagnostic, Parsed, Parser};
use proc_macro::TokenStream;
use proc_macro_error::{abort_call_site, emit_call_site_error, proc_macro_error};
use std::env;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Error, Ident, LitBool, LitStr, Result, Token};

struct Args {
    schema: PathBuf,
    includes: Vec<PathBuf>,
    options: Options,
    warnings_as_errors: bool,
    suppress_warnings: bool,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut schema = PathBuf::from(input.parse::<LitStr>()?.value());
        if !schema.is_absolute() {
            let mut root = env::var("CARGO_MANIFEST_DIR")
                .map(PathBuf::from)
                .unwrap_or_default();
            root.push(schema);
            schema = root;
        }

        let mut args = Args {
            schema,
            includes: Vec::new(),
            options: Options::default(),
            warnings_as_errors: false,
            suppress_warnings: false,
        };
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }

            let opt: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if opt == "include" {
                let path = PathBuf::from(input.parse::<LitStr>()?.value());
                args.includes.push(path);
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
        }

        Ok(args)
    }
}

#[proc_macro_error]
#[proc_macro]
pub fn generate(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);

    let mut parser = Parser::new();
    for include in args.includes {
        parser.add_schema_path(include);
    }
    let parsed = parser.parse(&args.schema);

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
                // Work-around a bug in emit_call_site_warning!(..)
                proc_macro_error::diagnostic!(
                    proc_macro2::Span::call_site(),
                    proc_macro_error::Level::Warning,
                    msg
                )
                .emit();
            }
        }
    }

    if !parsed.errors().is_empty() || (args.warnings_as_errors && !parsed.warnings().is_empty()) {
        abort_call_site!("there were Aldrin schema errors");
    }

    let gen = Generator::new(&args.options, &parsed);
    let rust_options = RustOptions::new();
    let output = match gen.generate_rust(&rust_options) {
        Ok(output) => output,
        Err(e) => panic!("{}", e),
    };

    let module = format!(
        "pub mod {} {{ {} const _: &[u8] = include_bytes!(\"{}\"); }}",
        output.module_name,
        output.module_content,
        args.schema.display()
    );
    module.parse().unwrap()
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
