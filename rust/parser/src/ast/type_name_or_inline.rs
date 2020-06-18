use super::TypeName;
use crate::grammar::Rule;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub enum TypeNameOrInline {
    TypeName(TypeName),
}

impl TypeNameOrInline {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::type_name_or_inline);

        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::type_name => TypeNameOrInline::TypeName(TypeName::parse(pair)),
            _ => unreachable!(),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            TypeNameOrInline::TypeName(t) => t.span(),
        }
    }
}
