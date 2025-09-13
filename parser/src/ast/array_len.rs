use super::{LitInt, NamedRef};
use crate::error::{
    ConstIntNotFound, ExpectedConstIntFoundService, ExpectedConstIntFoundString,
    ExpectedConstIntFoundType, ExpectedConstIntFoundUuid, InvalidArrayLen,
};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::Span;
use pest::iterators::Pair;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ArrayLen {
    span: Span,
    value: ArrayLenValue,
}

impl ArrayLen {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::array_len);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        let value = ArrayLenValue::parse(pair);

        Self { span, value }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        InvalidArrayLen::validate(self, validate);

        self.value.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn value(&self) -> &ArrayLenValue {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub enum ArrayLenValue {
    Literal(LitInt),
    Ref(NamedRef),
}

impl ArrayLenValue {
    fn parse(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::lit_int => Self::Literal(LitInt::parse(pair)),
            Rule::named_ref => Self::Ref(NamedRef::parse(pair)),
            _ => unreachable!(),
        }
    }

    fn validate(&self, validate: &mut Validate) {
        match self {
            Self::Literal(_) => {}

            Self::Ref(ty) => {
                ConstIntNotFound::validate(ty, validate);
                ExpectedConstIntFoundService::validate(ty, validate);
                ExpectedConstIntFoundString::validate(ty, validate);
                ExpectedConstIntFoundType::validate(ty, validate);
                ExpectedConstIntFoundUuid::validate(ty, validate);

                ty.validate(validate);
            }
        }
    }
}

impl fmt::Display for ArrayLenValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Literal(lit) => lit.value().fmt(f),
            Self::Ref(named_ref) => named_ref.kind().fmt(f),
        }
    }
}
