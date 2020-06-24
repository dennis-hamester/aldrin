#![deny(intra_doc_link_resolution_failure)]

extern crate proc_macro;

use aldrin_codegen::{Generator, Options, RustOptions};
use aldrin_parser::{Diagnostic, Parser};
use proc_macro::TokenStream;
use std::env;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Error, Ident, LitBool, LitStr, Result, Token};

struct Args {
    schema: PathBuf,
    includes: Vec<PathBuf>,
    options: Options,
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

        let mut includes = Vec::new();
        let mut options = Options::default();
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }

            let opt: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if opt == "include" {
                let path = PathBuf::from(input.parse::<LitStr>()?.value());
                includes.push(path);
            } else if opt == "client" {
                options.client = input.parse::<LitBool>()?.value;
            } else if opt == "server" {
                options.server = input.parse::<LitBool>()?.value;
            } else {
                return Err(Error::new_spanned(opt, "invalid option"));
            }
        }

        Ok(Args {
            schema,
            includes,
            options,
        })
    }
}

#[proc_macro]
pub fn generate(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);

    let mut parser = Parser::new();
    for include in args.includes {
        parser.add_schema_path(include);
    }
    let parsed = parser.parse(&args.schema);

    if !parsed.errors().is_empty() {
        let mut panic_msg = String::new();

        for error in parsed.errors() {
            let formatted = error.format(&parsed);
            panic_msg.push_str(&formatted.to_string());
            panic_msg.push('\n');
        }

        for warning in parsed.warnings() {
            let formatted = warning.format(&parsed);
            panic_msg.push_str(&formatted.to_string());
            panic_msg.push('\n');
        }

        panic!(panic_msg);
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
