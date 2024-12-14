use super::{Ident, LitPosInt, LitUuid, TypeNameOrInline};
use crate::error::{
    DuplicateEventId, DuplicateFunctionId, DuplicateServiceItem, InvalidEventId, InvalidFunctionId,
    InvalidServiceUuid, InvalidServiceVersion,
};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::{NonCamelCaseService, NonSnakeCaseEvent, NonSnakeCaseFunction};
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ServiceDef {
    span: Span,
    name: Ident,
    uuid: LitUuid,
    ver: LitPosInt,
    items: Vec<ServiceItem>,
    fn_fallback: Option<FunctionFallbackDef>,
    ev_fallback: Option<EventFallbackDef>,
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

        let mut items = Vec::new();
        let mut fn_fallback = None;
        let mut ev_fallback = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::service_item => items.push(ServiceItem::parse(pair)),

                Rule::service_fallback => {
                    for pair in pair.into_inner() {
                        match pair.as_rule() {
                            Rule::fn_fallback => {
                                fn_fallback = Some(FunctionFallbackDef::parse(pair))
                            }

                            Rule::event_fallback => {
                                ev_fallback = Some(EventFallbackDef::parse(pair))
                            }

                            _ => unreachable!(),
                        }
                    }
                }

                Rule::tok_cur_close => break,
                _ => unreachable!(),
            }
        }

        Self {
            span,
            name,
            uuid,
            ver,
            items,
            fn_fallback,
            ev_fallback,
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
        DuplicateServiceItem::validate(self, validate);
        DuplicateFunctionId::validate(self, validate);
        DuplicateEventId::validate(self, validate);
        NonCamelCaseService::validate(self, validate);

        self.name.validate(validate);

        for item in &self.items {
            item.validate(validate);
        }

        if let Some(ref fn_fallback) = self.fn_fallback {
            fn_fallback.validate(validate);
        }

        if let Some(ref ev_fallback) = self.ev_fallback {
            ev_fallback.validate(validate);
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

    pub fn items(&self) -> &[ServiceItem] {
        &self.items
    }

    pub fn function_fallback(&self) -> Option<&FunctionFallbackDef> {
        self.fn_fallback.as_ref()
    }

    pub fn event_fallback(&self) -> Option<&EventFallbackDef> {
        self.ev_fallback.as_ref()
    }
}

#[derive(Debug, Clone)]
pub enum ServiceItem {
    Function(FunctionDef),
    Event(EventDef),
}

impl ServiceItem {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::service_item);
        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::fn_def => Self::Function(FunctionDef::parse(pair)),
            Rule::event_def => Self::Event(EventDef::parse(pair)),
            _ => unreachable!(),
        }
    }

    fn validate(&self, validate: &mut Validate) {
        match self {
            Self::Function(i) => i.validate(validate),
            Self::Event(i) => i.validate(validate),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Function(i) => i.span(),
            Self::Event(i) => i.span(),
        }
    }

    pub fn name(&self) -> &Ident {
        match self {
            Self::Function(i) => i.name(),
            Self::Event(i) => i.name(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    span: Span,
    name: Ident,
    id: LitPosInt,
    args: Option<FunctionPart>,
    ok: Option<FunctionPart>,
    err: Option<FunctionPart>,
}

impl FunctionDef {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::fn_def);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip @.

        let pair = pairs.next().unwrap();
        let id = LitPosInt::parse(pair);

        let mut args = None;
        let mut ok = None;
        let mut err = None;
        for pair in pairs {
            match pair.as_rule() {
                Rule::tok_cur_open => {}
                Rule::fn_args => args = Some(FunctionPart::parse(pair)),
                Rule::fn_ok => ok = Some(FunctionPart::parse(pair)),
                Rule::fn_err => err = Some(FunctionPart::parse(pair)),
                Rule::tok_cur_close | Rule::tok_term => break,
                _ => unreachable!(),
            }
        }

        Self {
            span,
            name,
            id,
            args,
            ok,
            err,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        NonSnakeCaseFunction::validate(self, validate);
        InvalidFunctionId::validate(self, validate);

        self.name.validate(validate);

        if let Some(ref args) = self.args {
            args.validate(validate);
        }

        if let Some(ref ok) = self.ok {
            ok.validate(validate);
        }

        if let Some(ref err) = self.err {
            err.validate(validate);
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn id(&self) -> &LitPosInt {
        &self.id
    }

    pub fn args(&self) -> Option<&FunctionPart> {
        self.args.as_ref()
    }

    pub fn ok(&self) -> Option<&FunctionPart> {
        self.ok.as_ref()
    }

    pub fn err(&self) -> Option<&FunctionPart> {
        self.err.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionPart {
    span: Span,
    part_type: TypeNameOrInline,
}

impl FunctionPart {
    fn parse(pair: Pair<Rule>) -> Self {
        assert!(
            (pair.as_rule() == Rule::fn_args)
                || (pair.as_rule() == Rule::fn_ok)
                || (pair.as_rule() == Rule::fn_err)
        );

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        pairs.next().unwrap(); // Skip keyword.
        pairs.next().unwrap(); // Skip =.

        let pair = pairs.next().unwrap();
        let part_type = TypeNameOrInline::parse(pair);

        Self { span, part_type }
    }

    fn validate(&self, validate: &mut Validate) {
        self.part_type.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn part_type(&self) -> &TypeNameOrInline {
        &self.part_type
    }
}

#[derive(Debug, Clone)]
pub struct EventDef {
    span: Span,
    name: Ident,
    id: LitPosInt,
    event_type: Option<TypeNameOrInline>,
}

impl EventDef {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::event_def);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip @.

        let pair = pairs.next().unwrap();
        let id = LitPosInt::parse(pair);

        let pair = pairs.next().unwrap();
        let event_type = match pair.as_rule() {
            Rule::tok_eq => {
                let pair = pairs.next().unwrap();
                Some(TypeNameOrInline::parse(pair))
            }
            Rule::tok_term => None,
            _ => unreachable!(),
        };

        Self {
            span,
            name,
            id,
            event_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        NonSnakeCaseEvent::validate(self, validate);
        InvalidEventId::validate(self, validate);

        self.name.validate(validate);
        if let Some(ref event_type) = self.event_type {
            event_type.validate(validate);
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn id(&self) -> &LitPosInt {
        &self.id
    }

    pub fn event_type(&self) -> Option<&TypeNameOrInline> {
        self.event_type.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionFallbackDef {
    span: Span,
    name: Ident,
}

impl FunctionFallbackDef {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::fn_fallback);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        Self { span, name }
    }

    fn validate(&self, validate: &mut Validate) {
        self.name.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct EventFallbackDef {
    span: Span,
    name: Ident,
}

impl EventFallbackDef {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::event_fallback);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        Self { span, name }
    }

    fn validate(&self, validate: &mut Validate) {
        self.name.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}
