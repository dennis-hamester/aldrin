#![no_main]

use aldrin_parser::{Parsed, Parser, Renderer};
use libfuzzer_sys::{fuzz_target, Corpus};
use std::collections::HashMap;
use std::fs;

fn gen_diagnostics(parsed: &Parsed) -> String {
    let renderer = Renderer::new(true, true, 100);
    let mut diag = String::new();

    for error in parsed.errors() {
        let rendered = renderer.render(error, parsed);
        diag.push_str(&rendered);
    }

    for warning in parsed.warnings() {
        let rendered = renderer.render(warning, parsed);
        diag.push_str(&rendered);
    }

    for warning in parsed.other_warnings() {
        let rendered = renderer.render(warning, parsed);
        diag.push_str(&rendered);
    }

    diag
}

fuzz_target!(|schemas: HashMap<String, String>| -> Corpus {
    if schemas.is_empty()
        || schemas
            .keys()
            .any(|s| s.is_empty() || s.contains(['/', '.']))
    {
        return Corpus::Reject;
    }

    let dir = tempfile::tempdir().unwrap();
    let mut main_schema = None;

    for (schema_name, schema) in &schemas {
        let mut path = dir.path().to_path_buf();
        path.push(schema_name);
        path.set_extension(".aldrin");

        let _ = fs::write(&path, schema);

        if main_schema.is_none() {
            main_schema = Some(path);
        }
    }

    let mut parser = Parser::new();
    parser.add_schema_path(dir.path());

    let parsed = parser.parse(main_schema.unwrap());
    let _ = gen_diagnostics(&parsed);

    Corpus::Keep
});
