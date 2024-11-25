use super::{LitPosInt, NamedRef};
use crate::error::{ConstIntNotFound, ExpectedConstIntFoundType};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::Span;
use pest::iterators::Pair;

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
    Literal(LitPosInt),
    Ref(NamedRef),
}

impl ArrayLenValue {
    fn parse(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::lit_pos_int => Self::Literal(LitPosInt::parse(pair)),
            Rule::named_ref => Self::Ref(NamedRef::parse(pair)),
            _ => unreachable!(),
        }
    }

    fn validate(&self, validate: &mut Validate) {
        match self {
            Self::Literal(_) => {}

            Self::Ref(ty) => {
                ConstIntNotFound::validate(ty, validate);
                ExpectedConstIntFoundType::validate(ty, validate);

                ty.validate(validate);
            }
        }
    }
}
