use crate::Color;
use aldrin_parser::diag::{Diagnostic, Formatted, Style};
use aldrin_parser::Parsed;
use once_cell::sync::Lazy;
use std::io::{Result, Write};
use termcolor::{Color as TermColor, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn style(fg: Option<TermColor>, bold: bool) -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(fg);
    spec.set_bold(bold);
    spec
}

static STYLE_REGULAR: Lazy<ColorSpec> = Lazy::new(|| style(None, false));
static STYLE_ERROR: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Red), true));
static STYLE_WARNING: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Yellow), true));
static STYLE_INFO: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Blue), true));
static STYLE_EMPHASIZED: Lazy<ColorSpec> = Lazy::new(|| style(None, true));
static STYLE_SEPARATOR: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Cyan), true));
static STYLE_LINE_NUMBER: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Cyan), true));

pub fn print_diagnostics(parsed: &Parsed, color: Color) -> Result<()> {
    let color_choice = match color {
        Color::Auto if atty::is(atty::Stream::Stderr) => ColorChoice::Auto,
        Color::Auto => ColorChoice::Never,
        Color::Always => ColorChoice::Always,
        Color::Never => ColorChoice::Never,
    };

    let mut stream = StandardStream::stderr(color_choice);
    stream.reset()?;

    for error in parsed.errors() {
        let formatted = error.format(parsed);
        print_formatted(&mut stream, &formatted)?;
        writeln!(stream)?;
    }

    for warning in parsed.warnings() {
        let formatted = warning.format(parsed);
        print_formatted(&mut stream, &formatted)?;
        writeln!(stream)?;
    }

    Ok(())
}

fn print_formatted<W>(w: &mut W, formatted: &Formatted) -> Result<()>
where
    W: WriteColor,
{
    for line in formatted {
        for (chunk, style) in line {
            let style = match style {
                Style::Regular => &STYLE_REGULAR,
                Style::Error => &STYLE_ERROR,
                Style::Warning => &STYLE_WARNING,
                Style::Info => &STYLE_INFO,
                Style::Emphasized => &STYLE_EMPHASIZED,
                Style::Separator => &STYLE_SEPARATOR,
                Style::LineNumber => &STYLE_LINE_NUMBER,
            };

            w.set_color(style)?;
            write!(w, "{}", chunk)?;
        }

        w.set_color(&STYLE_REGULAR)?;
        writeln!(w)?;
    }

    Ok(())
}
