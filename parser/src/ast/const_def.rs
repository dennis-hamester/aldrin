use super::{DocString, Ident, LitInt, LitString, LitUuid, Prelude};
use crate::error::{InvalidConstValue, InvalidEscapeCode};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::{BrokenDocLink, NonShoutySnakeCaseConst};
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ConstDef {
    span: Span,
    comment: Option<String>,
    doc: Vec<DocString>,
    name: Ident,
    value_span: Span,
    value: ConstValue,
}

impl ConstDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::const_def);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        pairs.next().unwrap(); // Skip keyword.

        let name = Ident::parse(pairs.next().unwrap());

        pairs.next().unwrap(); // Skip =.

        let value_pair = pairs.next().unwrap();
        let value_span = Span::from_pair(&value_pair);
        let value = ConstValue::parse(value_pair);

        Self {
            span,
            comment: prelude.take_comment().into(),
            doc: prelude.take_doc(),
            name,
            value_span,
            value,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        BrokenDocLink::validate(&self.doc, validate);
        InvalidEscapeCode::validate(self, validate);
        NonShoutySnakeCaseConst::validate(self, validate);

        self.name.validate(true, validate);
        self.value.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    pub fn doc(&self) -> &[DocString] {
        &self.doc
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
        pairs.next().unwrap(); // Skip (.
        let pair = pairs.next().unwrap();

        match rule {
            Rule::kw_u8 => Self::U8(LitInt::parse(pair)),
            Rule::kw_i8 => Self::I8(LitInt::parse(pair)),
            Rule::kw_u16 => Self::U16(LitInt::parse(pair)),
            Rule::kw_i16 => Self::I16(LitInt::parse(pair)),
            Rule::kw_u32 => Self::U32(LitInt::parse(pair)),
            Rule::kw_i32 => Self::I32(LitInt::parse(pair)),
            Rule::kw_u64 => Self::U64(LitInt::parse(pair)),
            Rule::kw_i64 => Self::I64(LitInt::parse(pair)),
            Rule::kw_string => Self::String(LitString::parse(pair)),
            Rule::kw_uuid => Self::Uuid(LitUuid::parse(pair)),
            _ => unreachable!(),
        }
    }

    fn validate(&self, validate: &mut Validate) {
        InvalidConstValue::validate(self, validate);
    }
}
