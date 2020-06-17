use crate::grammar::Rule;
use crate::issues::Issues;
use crate::warning::NonSnakeCaseSchemaName;
use crate::Span;
use heck::SnakeCase;
use pest::iterators::Pair;

#[derive(Debug)]
pub struct SchemaName {
    span: Span,
    value: String,
}

impl SchemaName {
    pub(crate) fn parse(pair: Pair<Rule>, issues: &mut Issues, is_definition: bool) -> Self {
        assert_eq!(pair.as_rule(), Rule::schema_name);

        let value = pair.as_str().to_owned();
        if is_definition {
            let snake_case = value.to_snake_case();
            if value != snake_case {
                issues.add_warning(NonSnakeCaseSchemaName::new(value.clone(), snake_case));
            }
        }

        SchemaName {
            span: Span::from_pair(&pair),
            value,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
