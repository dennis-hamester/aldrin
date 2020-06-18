use crate::ast::{ConstDef, Ident, StructDef};
use crate::validate::Validate;
use crate::Span;

#[derive(Debug, Clone)]
pub enum Definition {
    Const(ConstDef),
    Struct(StructDef),
}

impl Definition {
    pub(crate) fn validate(&self, validate: &mut Validate) {
        match self {
            Definition::Const(d) => d.validate(validate),
            Definition::Struct(d) => d.validate(validate),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Definition::Const(d) => d.span(),
            Definition::Struct(d) => d.span(),
        }
    }

    pub fn name(&self) -> &Ident {
        match self {
            Definition::Const(d) => d.name(),
            Definition::Struct(d) => d.name(),
        }
    }
}
