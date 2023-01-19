use super::{Ident, LitInt, LitString, LitUuid};
use crate::error::InvalidConstValue;
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::NonShoutySnakeCaseConst;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ConstDef {
    span: Span,
    name: Ident,
    value_span: Span,
    value: ConstValue,
}

impl ConstDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::const_def);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();
        pairs.next().unwrap(); // Skip keyword.

        let name = Ident::parse(pairs.next().unwrap());

        pairs.next().unwrap(); // Skip =.

        let value_pair = pairs.next().unwrap();
        let value_span = Span::from_pair(&value_pair);
        let value = ConstValue::parse(value_pair);

        ConstDef {
            span,
            name,
            value_span,
            value,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        NonShoutySnakeCaseConst::validate(self, validate);

        self.name.validate(validate);
        self.value.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn value_span(&self) -> Span {
        self.value_span
    }

    pub fn value(&self) -> &ConstValue {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub enum ConstValue {
    U8(LitInt),
    I8(LitInt),
    U16(LitInt),
    I16(LitInt),
    U32(LitInt),
    I32(LitInt),
    U64(LitInt),
    I64(LitInt),
    String(LitString),
    Uuid(LitUuid),
}

impl ConstValue {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::const_value);
        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();

        let rule = pair.as_rule();
        let mut pairs = pair.into_inner();
        pairs.next().unwrap(); // Skip type keyword.
        pairs.next().unwrap(); // Skip (.
        let pair = pairs.next().unwrap();

        match rule {
            Rule::const_u8 => ConstValue::U8(LitInt::parse(pair)),
            Rule::const_i8 => ConstValue::I8(LitInt::parse(pair)),
            Rule::const_u16 => ConstValue::U16(LitInt::parse(pair)),
            Rule::const_i16 => ConstValue::I16(LitInt::parse(pair)),
            Rule::const_u32 => ConstValue::U32(LitInt::parse(pair)),
            Rule::const_i32 => ConstValue::I32(LitInt::parse(pair)),
            Rule::const_u64 => ConstValue::U64(LitInt::parse(pair)),
            Rule::const_i64 => ConstValue::I64(LitInt::parse(pair)),
            Rule::const_string => ConstValue::String(LitString::parse(pair)),
            Rule::const_uuid => ConstValue::Uuid(LitUuid::parse(pair)),
            _ => unreachable!(),
        }
    }

    fn validate(&self, validate: &mut Validate) {
        InvalidConstValue::validate(self, validate);
    }
}
