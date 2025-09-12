use crate::error::InvalidIdent;
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::ReservedIdent;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct Ident {
    span: Span,
    value: String,
}

impl Ident {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::ident);

        Self {
            span: Span::from_pair(&pair),
            value: pair.as_str().to_owned(),
        }
    }

    pub(crate) fn validate(&self, is_def: bool, validate: &mut Validate) {
        if is_def {
            InvalidIdent::validate(self, validate);
            ReservedIdent::validate(self, validate);
        }
    }

    pub(crate) fn is_valid(ident: &str) -> bool {
        !ident.is_empty()
            && ident.chars().enumerate().all(|(i, c)| {
                (c == '_')
                    || if i == 0 {
                        c.is_ascii_alphabetic()
                    } else {
                        c.is_ascii_alphanumeric()
                    }
            })
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
