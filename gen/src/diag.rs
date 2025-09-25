use aldrin_parser::{Diagnostic, Error, Parser, Renderer};
use anstream::eprintln;

pub(crate) fn print_diagnostics(parser: &Parser) {
    let renderer = Renderer::new(true, true, get_termwidth());
    let mut first = true;

    for error in parser.errors() {
        let rendered = error.render(&renderer, parser);
        print_newline(&mut first);
        eprintln!("{rendered}");
    }

    for warning in parser.warnings() {
        let rendered = warning.render(&renderer, parser);
        print_newline(&mut first);
        eprintln!("{rendered}");
    }

    for warning in parser.other_warnings() {
        let rendered = warning.render(&renderer, parser);
        print_newline(&mut first);
        eprintln!("{rendered}");
    }
}

#[allow(single_use_lifetimes)]
pub(crate) fn print_errors<'a>(parser: &Parser, errs: impl IntoIterator<Item = &'a Error>) {
    let renderer = Renderer::new(true, true, get_termwidth());
    let mut first = true;

    for error in errs {
        let rendered = error.render(&renderer, parser);
        print_newline(&mut first);
        eprintln!("{rendered}");
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

fn print_newline(first: &mut bool) {
    if *first {
        *first = false;
    } else {
        eprintln!();
    }
}
