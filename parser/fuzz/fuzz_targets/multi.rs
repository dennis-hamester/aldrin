#![no_main]

mod diag;

use aldrin_parser::{Formatter, MemoryResolver, Parser};
use libfuzzer_sys::{Corpus, fuzz_target};
use std::collections::HashMap;

fuzz_target!(|schemas: HashMap<String, String>| -> Corpus {
    if schemas.is_empty() {
        return Corpus::Reject;
    }

    let mut schemas = schemas.into_iter();
    let (name, source) = schemas.next().unwrap();

    let mut resolver = MemoryResolver::new(name, Ok(source));

    for (name, source) in schemas {
        resolver.add(name, Ok(source));
    }

    let parser = Parser::parse(resolver);
    let _ = diag::gen_diagnostics(&parser);

    if let Ok(fmt) = Formatter::new(&parser) {
        fmt.to_string();
    }

    Corpus::Keep
});
