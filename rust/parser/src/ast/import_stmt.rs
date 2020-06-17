use super::SchemaName;
use crate::error::ImportNotFound;
use crate::grammar::Rule;
use crate::issues::Issues;
use crate::validate::Validate;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ImportStmt {
    span: Span,
    schema_name: SchemaName,
}

impl ImportStmt {
    pub(crate) fn parse(pair: Pair<Rule>, issues: &mut Issues) -> Self {
        assert_eq!(pair.as_rule(), Rule::import_stmt);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();
        pairs.next().unwrap(); // Skip keyword

        let schema_name = SchemaName::parse(pairs.next().unwrap(), issues, false);

        ImportStmt { span, schema_name }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        ImportNotFound::validate(self, validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn schema_name(&self) -> &SchemaName {
        &self.schema_name
    }
}
