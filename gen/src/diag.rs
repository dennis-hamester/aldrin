use crate::Color;
use aldrin_parser::diag::{
    Context, Diagnostic, DiagnosticKind, EmptyContext, Formatted, Indicator, Info, Intro, Line,
    Location, SkippedContext,
};
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

static STYLE_NONE: Lazy<ColorSpec> = Lazy::new(|| style(None, false));
static STYLE_ERROR: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Red), true));
static STYLE_WARNING: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Yellow), true));
static STYLE_INFO: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Blue), true));
static STYLE_INTRO: Lazy<ColorSpec> = Lazy::new(|| style(None, true));
static STYLE_SEP: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Cyan), true));
static STYLE_LOC: Lazy<ColorSpec> = Lazy::new(|| style(None, false));
static STYLE_LINE_NUMBER: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Cyan), true));
static STYLE_SOURCE: Lazy<ColorSpec> = Lazy::new(|| style(None, false));
static STYLE_SKIP: Lazy<ColorSpec> = Lazy::new(|| style(Some(TermColor::Cyan), true));
static STYLE_INFO_KIND: Lazy<ColorSpec> = Lazy::new(|| style(None, true));
static STYLE_INFO_TEXT: Lazy<ColorSpec> = Lazy::new(|| style(None, false));

macro_rules! styled {
    ($w:expr, $s:expr, $($args:expr),+) => {
        $w.set_color(&$s)?;
        $(
            write!($w, "{}", $args)?;
        )+
    }
}

macro_rules! styledln {
    ($w:expr) => {
        $w.reset()?;
        writeln!($w)?;
    };

    ($w:expr, $s:expr, $($args:expr),+) => {
        $w.set_color(&$s)?;
        $(
            write!($w, "{}", $args)?;
        )+
        styledln!($w);
    };
}

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
    let style = match formatted.kind {
        DiagnosticKind::Error => &STYLE_ERROR,
        DiagnosticKind::Warning => &STYLE_WARNING,
    };

    for line in &formatted.lines {
        print_line(w, line, style)?;
    }

    Ok(())
}

fn print_line<W>(w: &mut W, line: &Line, style: &ColorSpec) -> Result<()>
where
    W: WriteColor,
{
    match line {
        Line::Intro(l) => print_intro(w, l, style),
        Line::MainLocation(l) => print_location(w, l),
        Line::InfoLocation(l) => print_location(w, l),
        Line::EmptyContext(l) => print_empty_context(w, l),
        Line::Context(l) => print_context(w, l),
        Line::SkippedContext(l) => print_skipped_context(w, l),
        Line::MainIndicator(l) => print_indicator(w, l, style),
        Line::InfoIndicator(l) => print_indicator(w, l, &STYLE_INFO),
        Line::Info(l) => print_info(w, l),
    }
}

fn print_intro<W>(w: &mut W, intro: &Intro, style: &ColorSpec) -> Result<()>
where
    W: WriteColor,
{
    styled!(w, style, intro.kind);
    styledln!(w, STYLE_INTRO, intro.sep, intro.reason);
    Ok(())
}

fn print_location<W>(w: &mut W, loc: &Location) -> Result<()>
where
    W: WriteColor,
{
    styled!(w, STYLE_NONE, loc.pad1);
    styled!(w, STYLE_SEP, loc.sep);
    styled!(w, STYLE_NONE, loc.pad2);
    styledln!(w, STYLE_LOC, loc.path.display(), loc.line_col);
    Ok(())
}

fn print_empty_context<W>(w: &mut W, ctx: &EmptyContext) -> Result<()>
where
    W: WriteColor,
{
    styled!(w, STYLE_NONE, ctx.pad);
    styledln!(w, STYLE_SEP, ctx.sep);
    Ok(())
}

fn print_context<W>(w: &mut W, ctx: &Context) -> Result<()>
where
    W: WriteColor,
{
    styled!(w, STYLE_NONE, ctx.pad1);
    styled!(w, STYLE_LINE_NUMBER, ctx.line);
    styled!(w, STYLE_NONE, ctx.pad2);
    styled!(w, STYLE_SEP, ctx.sep);
    styled!(w, STYLE_NONE, ctx.pad3);
    styledln!(w, STYLE_SOURCE, ctx.source);
    Ok(())
}

fn print_skipped_context<W>(w: &mut W, ctx: &SkippedContext) -> Result<()>
where
    W: WriteColor,
{
    styled!(w, STYLE_NONE, ctx.pad1);
    styled!(w, STYLE_SKIP, ctx.skip);
    styled!(w, STYLE_NONE, ctx.pad2);
    styledln!(w, STYLE_SEP, ctx.sep);
    Ok(())
}

fn print_indicator<W>(w: &mut W, ind: &Indicator, style: &ColorSpec) -> Result<()>
where
    W: WriteColor,
{
    styled!(w, STYLE_NONE, ind.pad1);
    styled!(w, STYLE_SEP, ind.sep);
    styled!(w, STYLE_NONE, ind.pad2);
    styled!(w, style, ind.indicator);
    styled!(w, STYLE_NONE, ind.pad3);
    styledln!(w, style, ind.text);
    Ok(())
}

fn print_info<W>(w: &mut W, info: &Info) -> Result<()>
where
    W: WriteColor,
{
    styled!(w, STYLE_NONE, info.pad1);
    styled!(w, STYLE_SEP, info.sep1);
    styled!(w, STYLE_NONE, info.pad2);
    styled!(w, STYLE_INFO_KIND, info.kind);
    styled!(w, STYLE_INFO_KIND, info.sep2);
    styledln!(w, STYLE_INFO_TEXT, info.text);
    Ok(())
}
