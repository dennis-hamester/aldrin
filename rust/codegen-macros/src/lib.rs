extern crate proc_macro;

use aldrin_codegen::rust::RustOptions;
use aldrin_codegen::{Generator, Options};
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

    let mut options = Options::new();
    options.client = true;
    options.server = true;

    let gen = match Generator::from_path(&args.schema, options) {
        Ok(gen) => gen,
        Err(e) => panic!("{}", e),
    };

    let output = match gen.generate_rust(RustOptions::new()) {
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
