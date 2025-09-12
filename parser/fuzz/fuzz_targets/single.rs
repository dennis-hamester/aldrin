#![no_main]

mod diag;

use aldrin_parser::{MemoryResolver, Parser};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|schema: String| {
    let parser = Parser::parse(MemoryResolver::new("schema", Ok(schema)));
    let _ = diag::gen_diagnostics(&parser);
});
