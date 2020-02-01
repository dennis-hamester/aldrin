// Copyright (c) 2020 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

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
