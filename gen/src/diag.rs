use aldrin_parser::{Diagnostic, Parser, Renderer};
use anstream::eprintln;

pub(crate) fn print_diagnostics(parser: &Parser) {
    let renderer = Renderer::new(true, true, get_termwidth());

    for error in parser.errors() {
        let rendered = error.render(&renderer, parser);
        eprintln!("{rendered}\n");
    }

    for warning in parser.warnings() {
        let rendered = warning.render(&renderer, parser);
        eprintln!("{rendered}\n");
    }

    for warning in parser.other_warnings() {
        let rendered = warning.render(&renderer, parser);
        eprintln!("{rendered}\n");
    }
}

fn get_termwidth() -> usize {
    const MIN_TERMWIDTH: usize = 20;
    const DEFAULT_TERMWIDTH: usize = 100;

    terminal_size::terminal_size()
        .map(|(size, _)| size.0 as usize)
        .unwrap_or(DEFAULT_TERMWIDTH)
        .max(MIN_TERMWIDTH)
}
