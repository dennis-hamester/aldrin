use super::{Ident, LitPosInt, LitUuid};
use crate::error::{InvalidServiceUuid, InvalidServiceVersion};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::NonCamelCaseService;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ServiceDef {
    span: Span,
    name: Ident,
    uuid: LitUuid,
    ver: LitPosInt,
}

impl ServiceDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::service_def);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();
        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip {.

        let pair = pairs.next().unwrap();
        let uuid = Self::parse_uuid(pair);

        let pair = pairs.next().unwrap();
        let ver = Self::parse_version(pair);

        ServiceDef {
            span,
            name,
            uuid,
            ver,
        }
    }

    fn parse_uuid(pair: Pair<Rule>) -> LitUuid {
        assert_eq!(pair.as_rule(), Rule::service_uuid);
        let mut pairs = pair.into_inner();
        pairs.next().unwrap(); // Skip keyword.
        pairs.next().unwrap(); // Skip =.
        let pair = pairs.next().unwrap();
        LitUuid::parse(pair)
    }

    fn parse_version(pair: Pair<Rule>) -> LitPosInt {
        assert_eq!(pair.as_rule(), Rule::service_version);
        let mut pairs = pair.into_inner();
        pairs.next().unwrap(); // Skip keyword.
        pairs.next().unwrap(); // Skip =.
        let pair = pairs.next().unwrap();
        LitPosInt::parse(pair)
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        InvalidServiceUuid::validate(self, validate);
        InvalidServiceVersion::validate(self, validate);

        if validate.is_main_schema() {
            NonCamelCaseService::validate(self, validate);
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn uuid(&self) -> &LitUuid {
        &self.uuid
    }

    pub fn version(&self) -> &LitPosInt {
        &self.ver
    }
}
