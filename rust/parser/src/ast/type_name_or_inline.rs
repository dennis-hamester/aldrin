use super::{InlineEnum, InlineStruct, TypeName};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub enum TypeNameOrInline {
    TypeName(TypeName),
    Struct(InlineStruct),
    Enum(InlineEnum),
}

impl TypeNameOrInline {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::type_name_or_inline);

        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::type_name => TypeNameOrInline::TypeName(TypeName::parse(pair)),
            Rule::struct_inline => TypeNameOrInline::Struct(InlineStruct::parse(pair)),
            Rule::enum_inline => TypeNameOrInline::Enum(InlineEnum::parse(pair)),
            _ => unreachable!(),
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        match self {
            TypeNameOrInline::TypeName(ty) => ty.validate(validate),
            TypeNameOrInline::Struct(s) => s.validate(validate),
            TypeNameOrInline::Enum(e) => e.validate(validate),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            TypeNameOrInline::TypeName(t) => t.span(),
            TypeNameOrInline::Struct(s) => s.span(),
            TypeNameOrInline::Enum(e) => e.span(),
        }
    }
}
