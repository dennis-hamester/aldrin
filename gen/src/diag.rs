use aldrin_parser::{Diagnostic, Parsed, Renderer};
use anstream::eprintln;

pub(crate) fn print_diagnostics(parsed: &Parsed) {
    let renderer = Renderer::new(true, true, get_termwidth());

    for error in parsed.errors() {
        let rendered = error.render(&renderer, parsed);
        eprintln!("{rendered}\n");
    }

    for warning in parsed.warnings() {
        let rendered = warning.render(&renderer, parsed);
        eprintln!("{rendered}\n");
    }

    for warning in parsed.other_warnings() {
        let rendered = warning.render(&renderer, parsed);
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
