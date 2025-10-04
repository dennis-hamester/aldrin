use super::{Comment, Ident, Prelude};
use crate::error::ImportNotFound;
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::UnusedImport;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ImportStmt {
    span: Span,
    comment: Vec<Comment>,
    schema_name: Ident,
}

impl ImportStmt {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::import_stmt);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        pairs.next().unwrap(); // Skip keyword

        let pair = pairs.next().unwrap();
        let schema_name = Ident::parse(pair);

        Self {
            span,
            comment: prelude.take_comment(),
            schema_name,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        ImportNotFound::validate(self, validate);
        UnusedImport::validate(self, validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    pub fn schema_name(&self) -> &Ident {
        &self.schema_name
    }
}
