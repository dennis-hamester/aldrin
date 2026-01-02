use super::Ident;
use crate::Span;
use crate::error::MissingImport;
use crate::grammar::Rule;
use crate::validate::Validate;
use pest::iterators::Pair;
use std::fmt;

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

    pub(crate) fn dummy_intern(ident: Ident) -> Self {
        Self {
            span: Span::dummy(),
            kind: NamedRefKind::Intern(ident),
        }
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

    pub fn schema(&self) -> Option<&Ident> {
        self.kind.schema()
    }

    pub fn ident(&self) -> &Ident {
        self.kind.ident()
    }
}

#[derive(Debug, Clone)]
pub enum NamedRefKind {
    Intern(Ident),
    Extern(Ident, Ident),
}

impl NamedRefKind {
    fn parse(pair: Pair<Rule>) -> Self {
        #[expect(clippy::wildcard_enum_match_arm)]
        match pair.as_rule() {
            Rule::ident => Self::Intern(Ident::parse(&pair)),

            Rule::external_ref => {
                let mut pairs = pair.into_inner();
                let pair = pairs.next().unwrap();
                let schema_name = Ident::parse(&pair);
                pairs.next().unwrap(); // Skip ::.
                let ident = Ident::parse(&pairs.next().unwrap());

                Self::Extern(schema_name, ident)
            }

            _ => unreachable!(),
        }
    }

    fn validate(&self, validate: &mut Validate) {
        match self {
            Self::Intern(ty) => {
                ty.validate(false, validate);
            }

            Self::Extern(schema, ty) => {
                MissingImport::validate(schema, validate);
                ty.validate(false, validate);
            }
        }
    }

    pub fn schema(&self) -> Option<&Ident> {
        match self {
            Self::Intern(_) => None,
            Self::Extern(schema, _) => Some(schema),
        }
    }

    pub fn ident(&self) -> &Ident {
        match self {
            Self::Intern(ident) | Self::Extern(_, ident) => ident,
        }
    }
}

impl fmt::Display for NamedRefKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Intern(ident) => write!(f, "{}", ident.value()),
            Self::Extern(schema, ident) => write!(f, "{}::{}", schema.value(), ident.value()),
        }
    }
}
