use crate::ast::{ConstDef, EnumDef, Ident, StructDef};
use crate::validate::Validate;
use crate::Span;

#[derive(Debug, Clone)]
pub enum Definition {
    Struct(StructDef),
    Enum(EnumDef),
    Const(ConstDef),
}

impl Definition {
    pub(crate) fn validate(&self, validate: &mut Validate) {
        match self {
            Definition::Struct(d) => d.validate(validate),
            Definition::Enum(d) => d.validate(validate),
            Definition::Const(d) => d.validate(validate),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Definition::Struct(d) => d.span(),
            Definition::Enum(d) => d.span(),
            Definition::Const(d) => d.span(),
        }
    }

    pub fn name(&self) -> &Ident {
        match self {
            Definition::Struct(d) => d.name(),
            Definition::Enum(d) => d.name(),
            Definition::Const(d) => d.name(),
        }
    }
}
