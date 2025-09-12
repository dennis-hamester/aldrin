use aldrin_parser::{Parser, Renderer};

pub(crate) fn gen_diagnostics(parser: &Parser) -> String {
    let renderer = Renderer::new(true, true, 100);
    let mut diag = String::new();

    for error in parser.errors() {
        let rendered = renderer.render(error, parser);
        diag.push_str(&rendered);
    }

    for warning in parser.warnings() {
        let rendered = renderer.render(warning, parser);
        diag.push_str(&rendered);
    }

    for warning in parser.other_warnings() {
        let rendered = renderer.render(warning, parser);
        diag.push_str(&rendered);
    }

    diag
}
