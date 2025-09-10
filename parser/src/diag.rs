//! Diagnostic information and formatting.
//!
//! This module primarily provides the [`Diagnostic`] trait, which is implemented by
//! [`Error`](crate::Error) and [`Warning`](crate::Warning).

use crate::{Parsed, Schema, Span};
use annotate_snippets::renderer::DecorStyle;
use annotate_snippets::{AnnotationKind, Group, Level, Snippet};
use std::borrow::Cow;
use std::slice;

/// Diagnostic information about an error or a warning.
pub trait Diagnostic {
    /// Kind of the diagnostic; either an error or a warning.
    fn kind(&self) -> DiagnosticKind;

    /// Name of the schema this diagnostic originated from.
    ///
    /// The schema name can be used to look up the [`Schema`] with [`Parsed::get_schema`].
    fn schema_name(&self) -> &str;

    /// Renders the diagnostic for printing.
    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String;
}

/// Error or warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticKind {
    /// Indicates an issue which prevents further processing.
    Error,

    /// Indicates an issue which doesn't prevent further processing.
    Warning,
}

/// Renderer used to format [`Diagnostic`s](Diagnostic).
#[derive(Debug, Clone)]
pub struct Renderer {
    inner: annotate_snippets::Renderer,
}

impl Renderer {
    pub fn new(color: bool, unicode: bool, term_width: usize) -> Self {
        let mut inner = if color {
            annotate_snippets::Renderer::styled()
        } else {
            annotate_snippets::Renderer::plain()
        };

        if unicode {
            inner = inner.decor_style(DecorStyle::Unicode);
        };

        inner = inner.term_width(term_width);

        Self { inner }
    }

    pub fn render(&self, diagnostic: &(impl Diagnostic + ?Sized), parsed: &Parsed) -> String {
        diagnostic.render(self, parsed)
    }

    pub(crate) fn error<'a>(&'a self, title: impl Into<Cow<'a, str>>) -> Report<'a> {
        Report::new(
            Group::with_title(Level::ERROR.primary_title(title)),
            &self.inner,
        )
    }

    pub(crate) fn warning<'a>(&'a self, title: impl Into<Cow<'a, str>>) -> Report<'a> {
        Report::new(
            Group::with_title(Level::WARNING.primary_title(title)),
            &self.inner,
        )
    }
}

pub(crate) struct Report<'a> {
    group: Group<'a>,
    renderer: &'a annotate_snippets::Renderer,
}

impl<'a> Report<'a> {
    fn new(group: Group<'a>, renderer: &'a annotate_snippets::Renderer) -> Self {
        Self { group, renderer }
    }

    pub(crate) fn render(&self) -> String {
        self.renderer.render(slice::from_ref(&self.group))
    }

    pub(crate) fn snippet(
        mut self,
        schema: &'a Schema,
        span: Span,
        label: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.group = self.group.element(
            Snippet::source(schema.source().unwrap())
                .path(schema.path().to_string_lossy())
                .annotation(
                    AnnotationKind::Primary
                        .span(span.from.index..span.to.index)
                        .label(Some(label)),
                ),
        );

        self
    }

    pub(crate) fn context(
        mut self,
        schema: &'a Schema,
        span: Span,
        label: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.group = self.group.element(
            Snippet::source(schema.source().unwrap())
                .path(schema.path().to_string_lossy())
                .annotation(
                    AnnotationKind::Context
                        .span(span.from.index..span.to.index)
                        .label(Some(label)),
                ),
        );

        self
    }

    pub(crate) fn help(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.group = self.group.element(Level::HELP.message(text));
        self
    }

    pub(crate) fn note(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.group = self.group.element(Level::NOTE.message(text));
        self
    }
}
