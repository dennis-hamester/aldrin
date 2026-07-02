use crate::{Parser, SchemaRef, Span};
use annotate_snippets::renderer::DecorStyle;
use annotate_snippets::{AnnotationKind, Group, Level, Snippet};
use std::borrow::Cow;
use std::slice;

/// Diagnostic information about an error or a warning.
pub trait Diagnostic {
    /// Returns whether this diagnostic is an error or warning.
    fn kind(&self) -> DiagnosticKind;

    /// Returns a reference to the schema this diagnostic originated from.
    ///
    /// The schema can be look up using [`Parser::schema()`](crate::Parser::schema).
    fn schema(&self) -> SchemaRef;

    /// Renders the diagnostic for printing.
    fn render(&self, renderer: &DiagnosticRenderer, parser: &Parser) -> String;
}

/// Error or warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// Indicates an issue which prevents further processing.
    Error,

    /// Indicates an issue which doesn't prevent further processing.
    Warning,
}

/// Formats [`Diagnostic`s](Diagnostic) for printing.
#[derive(Debug, Clone)]
pub struct DiagnosticRenderer {
    inner: annotate_snippets::Renderer,
}

impl DiagnosticRenderer {
    pub fn new(color: bool, unicode: bool, term_width: usize) -> Self {
        let mut inner = if color {
            annotate_snippets::Renderer::styled()
        } else {
            annotate_snippets::Renderer::plain()
        };

        if unicode {
            inner = inner.decor_style(DecorStyle::Unicode);
        }

        inner = inner.term_width(term_width);

        Self { inner }
    }

    pub fn render(&self, diagnostic: &(impl Diagnostic + ?Sized), parser: &Parser) -> String {
        diagnostic.render(self, parser)
    }

    pub(crate) fn error<'a>(
        &'a self,
        title: impl Into<Cow<'a, str>>,
        parser: &'a Parser,
    ) -> Report<'a> {
        Report::new(
            Group::with_title(Level::ERROR.primary_title(title)),
            &self.inner,
            parser,
        )
    }

    pub(crate) fn warning<'a>(
        &'a self,
        title: impl Into<Cow<'a, str>>,
        parser: &'a Parser,
    ) -> Report<'a> {
        Report::new(
            Group::with_title(Level::WARNING.primary_title(title)),
            &self.inner,
            parser,
        )
    }
}

pub(crate) struct Report<'a> {
    group: Group<'a>,
    renderer: &'a annotate_snippets::Renderer,
    parser: &'a Parser,
}

impl<'a> Report<'a> {
    fn new(
        group: Group<'a>,
        renderer: &'a annotate_snippets::Renderer,
        parser: &'a Parser,
    ) -> Self {
        Self {
            group,
            renderer,
            parser,
        }
    }

    pub(crate) fn render(&self) -> String {
        self.renderer.render(slice::from_ref(&self.group))
    }

    pub(crate) fn snippet(
        mut self,
        schema: SchemaRef,
        span: Span,
        label: impl Into<Cow<'a, str>>,
    ) -> Self {
        let schema = self.parser.schema(schema);

        self.group = self.group.element(
            Snippet::source(schema.source().unwrap())
                .path(schema.path())
                .annotation(
                    AnnotationKind::Primary
                        .span(span.start..span.end)
                        .label(Some(label)),
                ),
        );

        self
    }

    // pub(crate) fn snippet_with_context(
    //     mut self,
    //     schema: SchemaRef,
    //     main_span: Span,
    //     main_label: impl Into<Cow<'a, str>>,
    //     context_span: Span,
    //     context_label: impl Into<Cow<'a, str>>,
    // ) -> Self {
    //     let schema = self.parser.schema(schema);

    //     self.group = self.group.element(
    //         Snippet::source(schema.source().unwrap())
    //             .path(schema.path())
    //             .annotation(
    //                 AnnotationKind::Primary
    //                     .span(main_span.start..main_span.end)
    //                     .label(Some(main_label)),
    //             )
    //             .annotation(
    //                 AnnotationKind::Context
    //                     .span(context_span.start..context_span.end)
    //                     .label(Some(context_label)),
    //             ),
    //     );

    //     self
    // }

    // pub(crate) fn context(
    //     mut self,
    //     schema: SchemaRef,
    //     span: Span,
    //     label: impl Into<Cow<'a, str>>,
    // ) -> Self {
    //     let schema = self.parser.schema(schema);

    //     self.group = self.group.element(
    //         Snippet::source(schema.source().unwrap())
    //             .path(schema.path())
    //             .annotation(
    //                 AnnotationKind::Context
    //                     .span(span.start..span.end)
    //                     .label(Some(label)),
    //             ),
    //     );

    //     self
    // }

    pub(crate) fn help(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.group = self.group.element(Level::HELP.message(text));
        self
    }

    pub(crate) fn note(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.group = self.group.element(Level::NOTE.message(text));
        self
    }
}
