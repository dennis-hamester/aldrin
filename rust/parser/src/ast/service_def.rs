use super::{Ident, LitPosInt, LitUuid, TypeNameOrInline};
use crate::error::{
    DuplicateEventId, DuplicateFunctionId, DuplicateServiceItem, InvalidServiceUuid,
    InvalidServiceVersion,
};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::{NonCamelCaseService, NonSnakeCaseFunction};
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ServiceDef {
    span: Span,
    name: Ident,
    uuid: LitUuid,
    ver: LitPosInt,
    items: Vec<ServiceItem>,
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
        for pair in pairs {
            match pair.as_rule() {
                Rule::service_item => items.push(ServiceItem::parse(pair)),
                Rule::tok_cur_close => break,
                _ => unreachable!(),
            }
        }

        ServiceDef {
            span,
            name,
            uuid,
            ver,
            items,
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

        if validate.is_main_schema() {
            NonCamelCaseService::validate(self, validate);
        }

        for item in &self.items {
            item.validate(validate);
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
            Rule::fn_def => ServiceItem::Function(FunctionDef::parse(pair)),
            Rule::event_def => ServiceItem::Event(EventDef::parse(pair)),
            _ => unreachable!(),
        }
    }

    fn validate(&self, validate: &mut Validate) {
        match self {
            ServiceItem::Function(i) => i.validate(validate),
            ServiceItem::Event(i) => i.validate(validate),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            ServiceItem::Function(i) => i.span(),
            ServiceItem::Event(i) => i.span(),
        }
    }

    pub fn name(&self) -> &Ident {
        match self {
            ServiceItem::Function(i) => i.name(),
            ServiceItem::Event(i) => i.name(),
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

        FunctionDef {
            span,
            name,
            id,
            args,
            ok,
            err,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        if validate.is_main_schema() {
            NonSnakeCaseFunction::validate(self, validate);
        }

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
    opt: bool,
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

        let opt;
        let part_type;
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::kw_optional => {
                opt = true;
                let pair = pairs.next().unwrap();
                part_type = TypeNameOrInline::parse(pair);
            }
            Rule::type_name_or_inline => {
                opt = false;
                part_type = TypeNameOrInline::parse(pair);
            }
            _ => unreachable!(),
        }

        FunctionPart {
            span,
            opt,
            part_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        self.part_type.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn optional(&self) -> bool {
        self.opt
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
    event_type: Option<EventType>,
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
                Some(EventType::parse(pair))
            }
            Rule::tok_term => None,
            _ => unreachable!(),
        };

        EventDef {
            span,
            name,
            id,
            event_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
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

    pub fn event_type(&self) -> Option<&EventType> {
        self.event_type.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct EventType {
    span: Span,
    opt: bool,
    event_type: TypeNameOrInline,
}

impl EventType {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::event_type);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        let opt;
        let event_type;
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::kw_optional => {
                opt = true;
                let pair = pairs.next().unwrap();
                event_type = TypeNameOrInline::parse(pair);
            }
            Rule::type_name_or_inline => {
                opt = false;
                event_type = TypeNameOrInline::parse(pair);
            }
            _ => unreachable!(),
        }

        EventType {
            span,
            opt,
            event_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        self.event_type.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn optional(&self) -> bool {
        self.opt
    }

    pub fn event_type(&self) -> &TypeNameOrInline {
        &self.event_type
    }
}
