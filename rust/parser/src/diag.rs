//! Diagnostic information and formatting.
//!
//! This module primarily provides the [`Diagnostic`] trait, which is implemented by all
//! [errors](crate::error) and [warnings](crate::warning).

use crate::{Parsed, Position, Schema, Span};
use std::borrow::Cow;
use std::cmp;
use std::fmt;
use std::path::Path;

/// Diagnostic information about an error or a warning.
pub trait Diagnostic {
    /// Kind of the diagnostic; either an error or a warning.
    fn kind(&self) -> DiagnosticKind;

    /// Name of the schema this diagnostic originated from.
    ///
    /// The schema name can be used to look up the [`Schema`] with [`Parsed::get_schema`].
    fn schema_name(&self) -> &str;

    /// Formats the diagnostic for printing.
    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a>;
}

/// Error or warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticKind {
    /// Indicates an issue which prevents further processing.
    Error,

    /// Indicates an issue which doesn't prevent further processing.
    Warning,
}

/// Style of chunk in a formatted diagnostic.
///
/// This describes broadly what particular chunk in [`Formatted`] contains and how it should be
/// styled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Style {
    /// Regular / unstyled output.
    Regular,

    /// Output e.g. as red and bold.
    Error,

    /// Output e.g. as yellow and bold.
    Warning,

    /// Output e.g. as blue and bold.
    Info,

    /// Output e.g. as bold.
    Emphasized,

    /// Output e.g. green.
    Separator,

    /// Output e.g. green and bold.
    LineNumber,
}

/// A diagnostic formatted for printing.
///
/// A `Formatted` can be printing and converted to a string directly via it's
/// [`Display`](fmt::Display) implementation.
///
/// Alternatively, you can iterate over the individual [`Line`s](Line) and from there over pairs of
/// `&str` and [`Style`]. This allows you to apply custom styling (e.g. colors) to individual parts
/// of the diagnostic.
///
/// `Formatted` also provides a short one-line [`summary`](Formatted::summary).
///
/// # Example
///
/// ```
/// use aldrin_parser::{Diagnostic, Parser};
///
/// let parsed = Parser::new().parse("schemas/duplicate_id.aldrin");
/// let err = &parsed.errors()[0];
/// let formatted = err.format(&parsed);
///
/// // Print via Display:
/// eprintln!("{}", formatted);
///
/// // Print a one-line summary:
/// eprintln!("Error: {}.", formatted.summary());
///
/// // Print manually to apply styling to the output:
/// for line in &formatted {
///     for (chunk, style) in line {
///         // Apply style to chunk, e.g. colorize the output.
///         eprint!("{}", chunk);
///     }
///
///     eprintln!();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Formatted<'a> {
    kind: DiagnosticKind,
    intro: Line<'a>,
    lines: Vec<Line<'a>>,
}

#[allow(clippy::len_without_is_empty)]
impl<'a> Formatted<'a> {
    /// Kind of the diagnostic; error or warning.
    pub fn kind(&self) -> DiagnosticKind {
        self.kind
    }

    /// Short one-line summary.
    ///
    /// The summary begins with a lower-case letter and doesn't end with any punctuation.
    ///
    /// # Example
    ///
    /// ```
    /// # use aldrin_parser::{Diagnostic, Parser};
    /// # let parsed = Parser::new().parse("schemas/duplicate_id.aldrin");
    /// # let err = &parsed.errors()[0];
    /// # let formatted = err.format(&parsed);
    /// eprintln!("An issue occurred: {}.", formatted.summary());
    /// ```
    pub fn summary(&self) -> &str {
        &self.intro.chunks[2].0
    }

    /// Returns number of lines.
    pub fn len(&self) -> usize {
        self.lines.len() + 1
    }

    /// Returns an iterator over the lines.
    pub fn lines(&'a self) -> Lines<'a> {
        Lines {
            intro: &self.intro,
            lines: &self.lines,
            line: 0,
        }
    }
}

impl<'a> IntoIterator for &'a Formatted<'a> {
    type Item = &'a Line<'a>;
    type IntoIter = Lines<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines()
    }
}

impl<'a> fmt::Display for Formatted<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self {
            line.fmt(f)?;
        }

        Ok(())
    }
}

/// Iterator over the lines of a formatted diagnostic.
#[derive(Debug)]
pub struct Lines<'a> {
    intro: &'a Line<'a>,
    lines: &'a [Line<'a>],
    line: usize,
}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a Line<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.line == 0 {
            self.line += 1;
            Some(self.intro)
        } else if self.line <= self.lines.len() {
            let line = &self.lines[self.line - 1];
            self.line += 1;
            Some(line)
        } else {
            None
        }
    }
}

/// Line of a formatted diagnostic.
#[derive(Debug, Clone)]
pub struct Line<'a> {
    padding: Cow<'a, str>,
    chunks: Vec<(Cow<'a, str>, Style)>,
}

#[allow(clippy::len_without_is_empty)]
impl<'a> Line<'a> {
    /// Returns number of chunks in this line.
    pub fn len(&self) -> usize {
        self.chunks.len() + 1
    }

    /// Returns an iterator of chunks in this line.
    pub fn chunks(&'a self) -> Chunks<'a> {
        Chunks {
            line: self,
            chunk: 0,
        }
    }
}

impl<'a> IntoIterator for &'a Line<'a> {
    type Item = (&'a str, Style);
    type IntoIter = Chunks<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.chunks()
    }
}

impl<'a> fmt::Display for Line<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (chunk, _) in self {
            f.write_str(chunk)?;
        }

        writeln!(f)
    }
}

/// Iterator of chunks in a line.
#[derive(Debug, Clone)]
pub struct Chunks<'a> {
    line: &'a Line<'a>,
    chunk: usize,
}

impl<'a> Iterator for Chunks<'a> {
    type Item = (&'a str, Style);

    fn next(&mut self) -> Option<Self::Item> {
        if self.chunk == 0 {
            self.chunk += 1;
            Some((&self.line.padding, Style::Regular))
        } else if self.chunk <= self.line.chunks.len() {
            let chunk = &self.line.chunks[self.chunk - 1];
            self.chunk += 1;
            Some((&chunk.0, chunk.1))
        } else {
            None
        }
    }
}

pub(crate) struct Formatter<'a> {
    kind: DiagnosticKind,
    intro: Line<'a>,
    lines: Vec<UnpaddedLine<'a>>,
    padding: usize,
}

impl<'a> Formatter<'a> {
    pub fn new<D, S>(diagnostic: &'a D, summary: S) -> Self
    where
        D: Diagnostic,
        S: Into<Cow<'a, str>>,
    {
        let (kind, kind_style) = match diagnostic.kind() {
            DiagnosticKind::Error => ("error", Style::Error),
            DiagnosticKind::Warning => ("warning", Style::Warning),
        };
        let summary = summary.into();
        let sep = if summary.is_empty() { ":" } else { ": " };

        let intro = Line {
            padding: "".into(),
            chunks: vec![
                (kind.into(), kind_style),
                (sep.into(), Style::Emphasized),
                // Formatted::summary() requires this (and only this) to be the 3rd entry.
                (summary, Style::Emphasized),
            ],
        };

        Formatter {
            kind: diagnostic.kind(),
            intro,
            lines: Vec::new(),
            padding: 0,
        }
    }

    pub fn format(self) -> Formatted<'a> {
        let mut lines = Vec::with_capacity(self.lines.len());
        for line in self.lines {
            lines.push(Line {
                padding: gen_padding(self.padding - line.padding),
                chunks: line.chunks,
            });
        }

        Formatted {
            kind: self.kind,
            intro: self.intro,
            lines,
        }
    }

    pub fn block<S>(
        &mut self,
        schema: &'a Schema,
        location: Position,
        indicator: Span,
        text: S,
        is_main_block: bool,
    ) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
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
        let mut lines = indicator.lines(source).peekable();

        while let Some((line, span)) = lines.next() {
            let line = line.trim_end();

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

            let trimmed = line.trim_start();
            let diff = line.len() - trimmed.len();
            let (from, to) = if diff >= 8 {
                self.trimmed_context(span.from.line_col.line, trimmed);
                (
                    cmp::max(span.from.line_col.column.saturating_sub(diff), 1) + 3,
                    span.to.line_col.column + 3 - diff,
                )
            } else {
                self.context(span.from.line_col.line, line);
                (span.from.line_col.column - 1, span.to.line_col.column - 1)
            };

            if lines.peek().is_some() {
                self.indicator(from, to, "", is_main_block);
            } else {
                self.indicator(from, to, text, is_main_block);
                self.empty_context();
                break;
            }
        }

        self
    }

    pub fn main_block<S>(
        &mut self,
        schema: &'a Schema,
        location: Position,
        span: Span,
        text: S,
    ) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.block(schema, location, span, text, true)
    }

    pub fn info_block<S>(
        &mut self,
        schema: &'a Schema,
        location: Position,
        span: Span,
        text: S,
    ) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.block(schema, location, span, text, false)
    }

    pub fn location<P>(&mut self, path: P, location: Position, is_main_location: bool) -> &mut Self
    where
        P: AsRef<Path>,
    {
        if is_main_location {
            self.main_location(path, location)
        } else {
            self.info_location(path, location)
        }
    }

    pub fn main_location<P>(&mut self, path: P, location: Position) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self.location_impl(path, location, "-->")
    }

    pub fn info_location<P>(&mut self, path: P, location: Position) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self.location_impl(path, location, ":::")
    }

    fn location_impl<P, S>(&mut self, path: P, location: Position, sep: S) -> &mut Self
    where
        P: AsRef<Path>,
        S: Into<Cow<'a, str>>,
    {
        let location = format!(
            " {}:{}:{}",
            path.as_ref().display(),
            location.line_col.line,
            location.line_col.column
        );

        self.lines.push(UnpaddedLine::new(vec![
            (" ".into(), Style::Regular),
            (sep.into(), Style::Separator),
            (location.into(), Style::Regular),
        ]));

        self
    }

    pub fn empty_context(&mut self) -> &mut Self {
        self.lines.push(UnpaddedLine::new(vec![
            ("  ".into(), Style::Regular),
            ("|".into(), Style::Separator),
        ]));

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

        self.lines.push(UnpaddedLine::with_padding(
            line.len(),
            vec![
                (" ".into(), Style::Regular),
                (line.into(), Style::LineNumber),
                (" ".into(), Style::Regular),
                ("|".into(), Style::Separator),
                (" ".into(), Style::Regular),
                (source.into(), Style::Regular),
            ],
        ));

        self
    }

    pub fn trimmed_context<S>(&mut self, line: usize, source: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        let line = line.to_string();
        if line.len() > self.padding {
            self.padding = line.len();
        }

        self.lines.push(UnpaddedLine::with_padding(
            line.len(),
            vec![
                (" ".into(), Style::Regular),
                (line.into(), Style::LineNumber),
                (" ".into(), Style::Regular),
                ("|".into(), Style::Separator),
                (" ... ".into(), Style::Regular),
                (source.into(), Style::Regular),
            ],
        ));

        self
    }

    pub fn skipped_context(&mut self) -> &mut Self {
        let skip = "..";
        if skip.len() > self.padding {
            self.padding = skip.len();
        }

        self.lines.push(UnpaddedLine::with_padding(
            skip.len(),
            vec![
                (" ".into(), Style::Regular),
                (skip.into(), Style::LineNumber),
                (" ".into(), Style::Regular),
                ("|".into(), Style::Separator),
            ],
        ));

        self
    }

    pub fn indicator<S>(
        &mut self,
        from: usize,
        to: usize,
        text: S,
        is_main_indicator: bool,
    ) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        if is_main_indicator {
            self.main_indicator(from, to, text)
        } else {
            self.info_indicator(from, to, text)
        }
    }

    pub fn main_indicator<S>(&mut self, from: usize, to: usize, text: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        let style = match self.kind {
            DiagnosticKind::Error => Style::Error,
            DiagnosticKind::Warning => Style::Warning,
        };

        self.indicator_impl(from, text, gen_main_indicator(to - from), style)
    }

    pub fn info_indicator<S>(&mut self, from: usize, to: usize, text: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.indicator_impl(from, text, gen_info_indicator(to - from), Style::Info)
    }

    fn indicator_impl<S, I>(&mut self, from: usize, text: S, ind: I, style: Style) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
        I: Into<Cow<'a, str>>,
    {
        let mut line = UnpaddedLine::new(vec![
            ("  ".into(), Style::Regular),
            ("|".into(), Style::Separator),
            (gen_padding(from + 1), Style::Regular),
            (ind.into(), style),
        ]);

        let text = text.into();
        if !text.is_empty() {
            line.chunks.push((" ".into(), Style::Regular));
            line.chunks.push((text, style));
        }

        self.lines.push(line);
        self
    }

    pub fn note<S>(&mut self, text: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.info_impl("note", text)
    }

    pub fn help<S>(&mut self, text: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.info_impl("help", text)
    }

    fn info_impl<K, S>(&mut self, kind: K, text: S) -> &mut Self
    where
        K: Into<Cow<'a, str>>,
        S: Into<Cow<'a, str>>,
    {
        self.lines.push(UnpaddedLine::new(vec![
            ("  ".into(), Style::Regular),
            ("=".into(), Style::Separator),
            (" ".into(), Style::Regular),
            (kind.into(), Style::Emphasized),
            (":".into(), Style::Emphasized),
            (" ".into(), Style::Regular),
            (text.into(), Style::Regular),
        ]));

        self
    }
}

struct UnpaddedLine<'a> {
    padding: usize,
    chunks: Vec<(Cow<'a, str>, Style)>,
}

impl<'a> UnpaddedLine<'a> {
    fn new(chunks: Vec<(Cow<'a, str>, Style)>) -> Self {
        UnpaddedLine { padding: 0, chunks }
    }

    fn with_padding(padding: usize, chunks: Vec<(Cow<'a, str>, Style)>) -> Self {
        UnpaddedLine { padding, chunks }
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
