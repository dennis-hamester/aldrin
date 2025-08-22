use super::{DocString, Ident, LitInt, LitString, LitUuid};
use crate::error::InvalidConstValue;
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::NonShoutySnakeCaseConst;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ConstDef {
    span: Span,
    doc: Option<String>,
    name: Ident,
    value_span: Span,
    value: ConstValue,
}

impl ConstDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::const_def);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();

        let mut doc = DocString::new();
        for pair in &mut pairs {
            match pair.as_rule() {
                Rule::doc_string => doc.push(pair),
                Rule::kw_const => break,
                _ => unreachable!(),
            }
        }

        let name = Ident::parse(pairs.next().unwrap());

        pairs.next().unwrap(); // Skip =.

        let value_pair = pairs.next().unwrap();
        let value_span = Span::from_pair(&value_pair);
        let value = ConstValue::parse(value_pair);

        Self {
            span,
            doc: doc.into(),
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

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
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
            Rule::const_u8 => Self::U8(LitInt::parse(pair)),
            Rule::const_i8 => Self::I8(LitInt::parse(pair)),
            Rule::const_u16 => Self::U16(LitInt::parse(pair)),
            Rule::const_i16 => Self::I16(LitInt::parse(pair)),
            Rule::const_u32 => Self::U32(LitInt::parse(pair)),
            Rule::const_i32 => Self::I32(LitInt::parse(pair)),
            Rule::const_u64 => Self::U64(LitInt::parse(pair)),
            Rule::const_i64 => Self::I64(LitInt::parse(pair)),
            Rule::const_string => Self::String(LitString::parse(pair)),
            Rule::const_uuid => Self::Uuid(LitUuid::parse(pair)),
            _ => unreachable!(),
        }
    }

    fn validate(&self, validate: &mut Validate) {
        InvalidConstValue::validate(self, validate);
    }
}
