use super::{Warning, WarningKind};
use crate::ast::DocString;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
use crate::validate::Validate;
use crate::{LinkResolver, Parser, ResolveLinkError, Span};
use comrak::nodes::{NodeValue, Sourcepos};
use comrak::{Arena, BrokenLinkReference, Options, ResolvedReference};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct BrokenDocLink {
    schema_name: String,
    span: Span,
    error: String,
}

impl BrokenDocLink {
    pub(crate) fn validate(doc: &[DocString], validate: &mut Validate) {
        if doc.is_empty() {
            return;
        }

        let mut doc_string = String::new();

        for doc in doc {
            doc_string.push_str(doc.value_inner());
            doc_string.push('\n');
        }

        let mut options = Options::default();

        options.parse.broken_link_callback = Some(Arc::new(|link: BrokenLinkReference| {
            LinkResolver::convert_broken_link(link.original).map(|link| ResolvedReference {
                url: link.to_owned(),
                title: String::new(),
            })
        }));

        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &doc_string, &options);
        let link_resolver = validate.link_resolver();

        for node in root.descendants() {
            let data = node.data.borrow();

            let NodeValue::Link(ref link) = data.value else {
                continue;
            };

            if let Err(e) = link_resolver.resolve(&link.url) {
                validate.add_warning(Self::from_error(e, doc, data.sourcepos, validate));
            }
        }
    }

    fn from_error(
        e: ResolveLinkError,
        doc: &[DocString],
        pos: Sourcepos,
        validate: &Validate,
    ) -> Self {
        let span = Span {
            start: doc[pos.start.line - 1].span_inner().start + pos.start.column - 1,
            end: doc[pos.end.line - 1].span_inner().start + pos.end.column,
        };

        Self {
            schema_name: validate.schema_name().to_owned(),
            span,
            error: e.to_string(),
        }
    }
}

impl Diagnostic for BrokenDocLink {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn render(&self, renderer: &Renderer, parser: &Parser) -> String {
        let mut report = renderer.warning(format!("broken doc link: {}", self.error));

        if let Some(schema) = parser.get_schema(&self.schema_name) {
            report = report.snippet(schema, self.span, "");
        }

        report.render()
    }
}

impl From<BrokenDocLink> for Warning {
    fn from(w: BrokenDocLink) -> Self {
        Self {
            kind: WarningKind::BrokenDocLink(w),
        }
    }
}
