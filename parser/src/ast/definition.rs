use crate::ast::{ConstDef, EnumDef, Ident, ServiceDef, StructDef};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Definition {
    Struct(StructDef),
    Enum(EnumDef),
    Service(ServiceDef),
    Const(ConstDef),
}

impl Definition {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::def);
        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::struct_def => Self::Struct(StructDef::parse(pair)),
            Rule::enum_def => Self::Enum(EnumDef::parse(pair)),
            Rule::service_def => Self::Service(ServiceDef::parse(pair)),
            Rule::const_def => Self::Const(ConstDef::parse(pair)),
            _ => unreachable!(),
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        match self {
            Self::Struct(d) => d.validate(validate),
            Self::Enum(d) => d.validate(validate),
            Self::Service(d) => d.validate(validate),
            Self::Const(d) => d.validate(validate),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Struct(d) => d.span(),
            Self::Enum(d) => d.span(),
            Self::Service(d) => d.span(),
            Self::Const(d) => d.span(),
        }
    }

    pub fn name(&self) -> &Ident {
        match self {
            Self::Struct(d) => d.name(),
            Self::Enum(d) => d.name(),
            Self::Service(d) => d.name(),
            Self::Const(d) => d.name(),
        }
    }
}
