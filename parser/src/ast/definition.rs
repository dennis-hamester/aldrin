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
            Rule::struct_def => Definition::Struct(StructDef::parse(pair)),
            Rule::enum_def => Definition::Enum(EnumDef::parse(pair)),
            Rule::service_def => Definition::Service(ServiceDef::parse(pair)),
            Rule::const_def => Definition::Const(ConstDef::parse(pair)),
            _ => unreachable!(),
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        match self {
            Definition::Struct(d) => d.validate(validate),
            Definition::Enum(d) => d.validate(validate),
            Definition::Service(d) => d.validate(validate),
            Definition::Const(d) => d.validate(validate),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Definition::Struct(d) => d.span(),
            Definition::Enum(d) => d.span(),
            Definition::Service(d) => d.span(),
            Definition::Const(d) => d.span(),
        }
    }

    pub fn name(&self) -> &Ident {
        match self {
            Definition::Struct(d) => d.name(),
            Definition::Enum(d) => d.name(),
            Definition::Service(d) => d.name(),
            Definition::Const(d) => d.name(),
        }
    }
}
