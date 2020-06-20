use crate::{Position, Schema, Span};
use std::borrow::Cow;
use std::fmt;
use std::path::Path;

pub trait Diagnostic {
    fn kind(&self) -> DiagnosticKind;
    fn schema_name(&self) -> &str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticKind {
    Error,
    Warning,
}

#[derive(Debug)]
pub struct Formatted<'a> {
    pub kind: DiagnosticKind,
    pub lines: Vec<Line<'a>>,
}

impl<'a> fmt::Display for Formatted<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            line.fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Line<'a> {
    Intro(Intro<'a>),
    MainLocation(Location<'a>),
    InfoLocation(Location<'a>),
    EmptyContext(EmptyContext),
    Context(Context<'a>),
    SkippedContext(SkippedContext),
    MainIndicator(Indicator),
    InfoIndicator(Indicator),
    Info(Info<'a>),
}

impl<'a> fmt::Display for Line<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Line::Intro(l) => l.fmt(f),
            Line::MainLocation(l) => l.fmt(f),
            Line::InfoLocation(l) => l.fmt(f),
            Line::EmptyContext(l) => l.fmt(f),
            Line::Context(l) => l.fmt(f),
            Line::SkippedContext(l) => l.fmt(f),
            Line::MainIndicator(l) => l.fmt(f),
            Line::InfoIndicator(l) => l.fmt(f),
            Line::Info(l) => l.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct Intro<'a> {
    pub kind: &'static str,
    pub sep: &'static str,
    pub reason: Cow<'a, str>,
}

impl<'a> fmt::Display for Intro<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}{}{}", self.kind, self.sep, self.reason)
    }
}

#[derive(Debug)]
pub struct Location<'a> {
    pub pad1: Cow<'static, str>,
    pub sep: &'static str,
    pub pad2: &'static str,
    pub path: Cow<'a, Path>,
    pub line_col: String,
}

impl<'a> fmt::Display for Location<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let path = self.path.display();
        writeln!(
            f,
            "{}{}{}{}{}",
            self.pad1, self.sep, self.pad2, path, self.line_col
        )
    }
}

#[derive(Debug)]
pub struct EmptyContext {
    pub pad: Cow<'static, str>,
    pub sep: &'static str,
}

impl fmt::Display for EmptyContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}{}", self.pad, self.sep)
    }
}

#[derive(Debug)]
pub struct Context<'a> {
    pub pad1: Cow<'static, str>,
    pub line: String,
    pub pad2: &'static str,
    pub sep: &'static str,
    pub pad3: &'static str,
    pub source: Cow<'a, str>,
}

impl<'a> fmt::Display for Context<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{}{}{}{}{}{}",
            self.pad1, self.line, self.pad2, self.sep, self.pad3, self.source
        )
    }
}

#[derive(Debug)]
pub struct SkippedContext {
    pub pad1: Cow<'static, str>,
    pub skip: &'static str,
    pub pad2: &'static str,
    pub sep: &'static str,
}

impl fmt::Display for SkippedContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}{}{}{}", self.pad1, self.skip, self.pad2, self.sep)
    }
}

#[derive(Debug)]
pub struct Indicator {
    pub pad1: Cow<'static, str>,
    pub sep: &'static str,
    pub pad2: Cow<'static, str>,
    pub indicator: Cow<'static, str>,
}

impl fmt::Display for Indicator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{}{}{}{}",
            self.pad1, self.sep, self.pad2, self.indicator
        )
    }
}

#[derive(Debug)]
pub struct Info<'a> {
    pub pad1: Cow<'static, str>,
    pub sep1: &'static str,
    pub pad2: &'static str,
    pub kind: &'static str,
    pub sep2: &'static str,
    pub text: Cow<'a, str>,
}

impl<'a> fmt::Display for Info<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{}{}{}{}{}{}",
            self.pad1, self.sep1, self.pad2, self.kind, self.sep2, self.text
        )
    }
}

#[derive(Debug)]
pub struct Formatter<'a> {
    kind: DiagnosticKind,
    lines: Vec<Line<'a>>,
    padding: usize,
}

impl<'a> Formatter<'a> {
    pub fn new<S>(kind: DiagnosticKind, reason: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        let intro = {
            let kind = match kind {
                DiagnosticKind::Error => "error",
                DiagnosticKind::Warning => "warning",
            };
            let reason = reason.into();
            let sep = if reason.is_empty() { ":" } else { ": " };
            Intro { kind, sep, reason }
        };

        Formatter {
            kind,
            lines: vec![Line::Intro(intro)],
            padding: 0,
        }
    }

    pub fn error<S>(reason: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self::new(DiagnosticKind::Error, reason)
    }

    pub fn warning<S>(reason: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self::new(DiagnosticKind::Warning, reason)
    }

    pub fn format(mut self) -> Formatted<'a> {
        for line in &mut self.lines {
            match line {
                Line::Intro(_) => {}
                Line::MainLocation(l) | Line::InfoLocation(l) => {
                    l.pad1 = gen_padding(self.padding + 1)
                }
                Line::EmptyContext(l) => l.pad = gen_padding(self.padding + 2),
                Line::Context(l) => l.pad1 = gen_padding(self.padding - l.line.len() + 1),
                Line::SkippedContext(l) => l.pad1 = gen_padding(self.padding - l.skip.len() + 1),
                Line::MainIndicator(l) | Line::InfoIndicator(l) => {
                    l.pad1 = gen_padding(self.padding + 2)
                }
                Line::Info(l) => l.pad1 = gen_padding(self.padding + 2),
            }
        }

        Formatted {
            kind: self.kind,
            lines: self.lines,
        }
    }

    pub fn block(
        &mut self,
        schema: &'a Schema,
        location: Position,
        span: Span,
        is_main_block: bool,
    ) -> &mut Self {
        self.location(schema.path(), location, is_main_block);

        let source = match schema.source() {
            Some(source) => source,
            None => return self,
        };

        #[derive(PartialEq, Eq)]
        enum State {
            Normal,
            Skip,
        }

        self.empty_context();
        let mut state = State::Normal;
        let mut lines = span.lines(source).peekable();

        while let Some((line, span)) = lines.next() {
            if line.trim().is_empty() {
                if state == State::Skip {
                    continue;
                }

                if let Some((next, _)) = lines.peek() {
                    if next.trim().is_empty() {
                        state = State::Skip;
                        self.skipped_context();
                        self.empty_context();
                        continue;
                    }
                } else {
                    continue;
                }
            }

            state = State::Normal;

            self.context(span.from.line_col.line, line);
            self.indicator(
                span.from.line_col.column - 1,
                span.to.line_col.column - 1,
                is_main_block,
            );
        }

        self
    }

    pub fn main_block(&mut self, schema: &'a Schema, location: Position, span: Span) -> &mut Self {
        self.block(schema, location, span, true)
    }

    pub fn info_block(&mut self, schema: &'a Schema, location: Position, span: Span) -> &mut Self {
        self.block(schema, location, span, false)
    }

    pub fn location<P>(&mut self, path: P, location: Position, is_main_location: bool) -> &mut Self
    where
        P: Into<Cow<'a, Path>>,
    {
        if is_main_location {
            self.main_location(path, location)
        } else {
            self.info_location(path, location)
        }
    }

    pub fn main_location<P>(&mut self, path: P, location: Position) -> &mut Self
    where
        P: Into<Cow<'a, Path>>,
    {
        self.lines.push(Line::MainLocation(Location {
            pad1: "".into(),
            sep: "-->",
            pad2: " ",
            path: path.into(),
            line_col: format!(":{}:{}", location.line_col.line, location.line_col.column),
        }));
        self
    }

    pub fn info_location<P>(&mut self, path: P, location: Position) -> &mut Self
    where
        P: Into<Cow<'a, Path>>,
    {
        self.lines.push(Line::InfoLocation(Location {
            pad1: "".into(),
            sep: ":::",
            pad2: " ",
            path: path.into(),
            line_col: format!(":{}:{}", location.line_col.line, location.line_col.column),
        }));
        self
    }

    pub fn empty_context(&mut self) -> &mut Self {
        self.lines.push(Line::EmptyContext(EmptyContext {
            pad: "".into(),
            sep: "|",
        }));
        self
    }

    pub fn context<S>(&mut self, line: usize, source: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        let line = line.to_string();
        if line.len() > self.padding {
            self.padding = line.len();
        }

        self.lines.push(Line::Context(Context {
            pad1: "".into(),
            line,
            pad2: " ",
            sep: "|",
            pad3: " ",
            source: source.into(),
        }));

        self
    }

    pub fn skipped_context(&mut self) -> &mut Self {
        let skip = "..";
        if skip.len() > self.padding {
            self.padding = skip.len();
        }

        self.lines.push(Line::SkippedContext(SkippedContext {
            pad1: "".into(),
            skip,
            pad2: " ",
            sep: "|",
        }));

        self
    }

    pub fn indicator(&mut self, from: usize, to: usize, is_main_indicator: bool) -> &mut Self {
        if is_main_indicator {
            self.main_indicator(from, to)
        } else {
            self.info_indicator(from, to)
        }
    }

    pub fn main_indicator(&mut self, from: usize, to: usize) -> &mut Self {
        self.lines.push(Line::MainIndicator(Indicator {
            pad1: "".into(),
            sep: "|",
            pad2: gen_padding(from + 1),
            indicator: gen_main_indicator(to - from),
        }));
        self
    }

    pub fn info_indicator(&mut self, from: usize, to: usize) -> &mut Self {
        self.lines.push(Line::InfoIndicator(Indicator {
            pad1: "".into(),
            sep: "|",
            pad2: gen_padding(from + 1),
            indicator: gen_info_indicator(to - from),
        }));
        self
    }

    fn info<S>(&mut self, sep: &'static str, kind: &'static str, text: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        let (sep2, text) = {
            let text = text.into();
            if text.is_empty() {
                ("", text)
            } else {
                (": ", text)
            }
        };

        self.lines.push(Line::Info(Info {
            pad1: "".into(),
            sep1: sep,
            pad2: " ",
            kind,
            sep2,
            text,
        }));

        self
    }

    pub fn note<S>(&mut self, text: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.info("=", "info", text)
    }

    pub fn help<S>(&mut self, text: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.info("=", "help", text)
    }
}

fn gen_padding(size: usize) -> Cow<'static, str> {
    const PADDING: &str = "                                                                ";
    if size < PADDING.len() {
        PADDING[..size].into()
    } else {
        use std::iter::{repeat, FromIterator};
        String::from_iter(repeat(' ').take(size)).into()
    }
}

fn gen_main_indicator(size: usize) -> Cow<'static, str> {
    const INDICATOR: &str = "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^";
    if size < INDICATOR.len() {
        INDICATOR[..size].into()
    } else {
        use std::iter::{repeat, FromIterator};
        String::from_iter(repeat('^').take(size)).into()
    }
}

fn gen_info_indicator(size: usize) -> Cow<'static, str> {
    const INDICATOR: &str = "----------------------------------------------------------------";
    if size < INDICATOR.len() {
        INDICATOR[..size].into()
    } else {
        use std::iter::{repeat, FromIterator};
        String::from_iter(repeat('-').take(size)).into()
    }
}
