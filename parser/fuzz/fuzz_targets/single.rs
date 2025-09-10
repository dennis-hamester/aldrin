#![no_main]

use aldrin_parser::{Parsed, Parser, Renderer};
use libfuzzer_sys::fuzz_target;
use std::io::Write;
use tempfile::NamedTempFile;

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

fuzz_target!(|schema: String| {
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(schema.as_bytes()).unwrap();
    file.flush().unwrap();

    let parser = Parser::new();
    let parsed = parser.parse(file.path());
    let _ = gen_diagnostics(&parsed);
});
