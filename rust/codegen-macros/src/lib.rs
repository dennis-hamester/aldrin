#![deny(intra_doc_link_resolution_failure)]

extern crate proc_macro;

use aldrin_codegen::{Generator, Options, RustOptions};
use aldrin_parser::{Diagnostic, Parser};
use proc_macro::TokenStream;
use std::env;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitStr, Result};

struct Args {
    schema: PathBuf,
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

        let args = Args { schema };

        Ok(args)
    }
}

#[proc_macro]
pub fn generate(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);

    let parser = Parser::new();
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

    let mut options = Options::new();
    options.client = true;
    options.server = true;

    let rust_options = RustOptions::new();

    let gen = Generator::new(&options, &parsed);
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
