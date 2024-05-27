use aldrin_parser::diag::{Diagnostic, Formatted, Style};
use aldrin_parser::Parsed;
use anstream::{eprint, eprintln};
use anstyle::{AnsiColor, Color, Style as AnStyle};

const fn style(fg: Option<AnsiColor>, bold: bool) -> AnStyle {
    let mut style = AnStyle::new();

    if let Some(fg) = fg {
        style = style.fg_color(Some(Color::Ansi(fg)));
    }

    if bold {
        style = style.bold();
    }

    style
}

const STYLE_REGULAR: AnStyle = style(None, false);
const STYLE_ERROR: AnStyle = style(Some(AnsiColor::Red), true);
const STYLE_WARNING: AnStyle = style(Some(AnsiColor::Yellow), true);
const STYLE_INFO: AnStyle = style(Some(AnsiColor::Blue), true);
const STYLE_EMPHASIZED: AnStyle = style(None, true);
const STYLE_SEPARATOR: AnStyle = style(Some(AnsiColor::Cyan), true);
const STYLE_LINE_NUMBER: AnStyle = style(Some(AnsiColor::Cyan), true);

pub fn print_diagnostics(parsed: &Parsed) {
    for error in parsed.errors() {
        let formatted = error.format(parsed);
        print_formatted(&formatted);
        eprintln!();
    }

    for warning in parsed.warnings() {
        let formatted = warning.format(parsed);
        print_formatted(&formatted);
        eprintln!();
    }

    for warning in parsed.other_warnings() {
        let formatted = warning.format(parsed);
        print_formatted(&formatted);
        eprintln!();
    }
}

fn print_formatted(formatted: &Formatted) {
    for line in formatted {
        for (chunk, style) in line {
            let style = match style {
                Style::Regular => STYLE_REGULAR,
                Style::Error => STYLE_ERROR,
                Style::Warning => STYLE_WARNING,
                Style::Info => STYLE_INFO,
                Style::Emphasized => STYLE_EMPHASIZED,
                Style::Separator => STYLE_SEPARATOR,
                Style::LineNumber => STYLE_LINE_NUMBER,
            };

            eprint!("{style}{chunk}{style:#}");
        }

        eprintln!();
    }
}
