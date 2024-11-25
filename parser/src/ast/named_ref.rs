use super::{Ident, SchemaName};
use crate::error::{ExternTypeNotFound, MissingImport, TypeNotFound};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct NamedRef {
    span: Span,
    kind: NamedRefKind,
}

impl NamedRef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::named_ref);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        let kind = NamedRefKind::parse(pair);

        Self { span, kind }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        self.kind.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn kind(&self) -> &NamedRefKind {
        &self.kind
    }

    pub fn schema(&self) -> Option<&SchemaName> {
        self.kind.schema()
    }

    pub fn ident(&self) -> &Ident {
        self.kind.ident()
    }
}

#[derive(Debug, Clone)]
pub enum NamedRefKind {
    Intern(Ident),
    Extern(SchemaName, Ident),
}

impl NamedRefKind {
    fn parse(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::ident => Self::Intern(Ident::parse(pair)),

            Rule::external_ref => {
                let mut pairs = pair.into_inner();
                let pair = pairs.next().unwrap();
                let schema_name = SchemaName::parse(pair);
                pairs.next().unwrap(); // Skip ::.
                let ident = Ident::parse(pairs.next().unwrap());

                Self::Extern(schema_name, ident)
            }

            _ => unreachable!(),
        }
    }

    fn validate(&self, validate: &mut Validate) {
        match self {
            Self::Intern(ty) => {
                TypeNotFound::validate(ty, validate);
            }

            Self::Extern(schema, ty) => {
                MissingImport::validate(schema, validate);
                ExternTypeNotFound::validate(schema, ty, validate);
            }
        }
    }

    pub fn schema(&self) -> Option<&SchemaName> {
        match self {
            Self::Intern(_) => None,
            Self::Extern(schema, _) => Some(schema),
        }
    }

    pub fn ident(&self) -> &Ident {
        match self {
            Self::Intern(ident) => ident,
            Self::Extern(_, ident) => ident,
        }
    }
}
