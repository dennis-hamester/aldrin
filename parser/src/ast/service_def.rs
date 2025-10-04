use super::{Comment, DocString, Ident, LitInt, LitUuid, Prelude, TypeNameOrInline};
use crate::error::{
    DuplicateEventId, DuplicateFunctionId, DuplicateServiceItem, InvalidEventId, InvalidFunctionId,
    InvalidServiceVersion,
};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::{BrokenDocLink, NonCamelCaseService, NonSnakeCaseEvent, NonSnakeCaseFunction};
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ServiceDef {
    span: Span,
    comment: Vec<Comment>,
    doc: Vec<DocString>,
    name: Ident,
    uuid_comment: Vec<Comment>,
    uuid: LitUuid,
    ver_comment: Vec<Comment>,
    ver: LitInt,
    items: Vec<ServiceItem>,
    fn_fallback: Option<FunctionFallback>,
    ev_fallback: Option<EventFallback>,
}

impl ServiceDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::service_def);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip {.

        let pair = pairs.next().unwrap();
        let (uuid_comment, uuid) = Self::parse_uuid(pair);

        let pair = pairs.next().unwrap();
        let (ver_comment, ver) = Self::parse_version(pair);

        let mut items = Vec::new();
        let mut fn_fallback = None;
        let mut ev_fallback = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::service_item => items.push(ServiceItem::parse(pair)),

                Rule::service_fallback => {
                    for pair in pair.into_inner() {
                        match pair.as_rule() {
                            Rule::fn_fallback => fn_fallback = Some(FunctionFallback::parse(pair)),
                            Rule::event_fallback => ev_fallback = Some(EventFallback::parse(pair)),
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
            comment: prelude.take_comment(),
            doc: prelude.take_doc(),
            name,
            uuid_comment,
            uuid,
            ver_comment,
            ver,
            items,
            fn_fallback,
            ev_fallback,
        }
    }

    fn parse_uuid(pair: Pair<Rule>) -> (Vec<Comment>, LitUuid) {
        assert_eq!(pair.as_rule(), Rule::service_uuid);

        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        pairs.next().unwrap(); // Skip keyword.
        pairs.next().unwrap(); // Skip =.

        let pair = pairs.next().unwrap();
        (prelude.take_comment(), LitUuid::parse(pair))
    }

    fn parse_version(pair: Pair<Rule>) -> (Vec<Comment>, LitInt) {
        assert_eq!(pair.as_rule(), Rule::service_version);

        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        pairs.next().unwrap(); // Skip keyword.
        pairs.next().unwrap(); // Skip =.

        let pair = pairs.next().unwrap();
        (prelude.take_comment(), LitInt::parse(pair))
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        BrokenDocLink::validate(&self.doc, validate);
        InvalidServiceVersion::validate(self, validate);
        DuplicateServiceItem::validate(self, validate);
        DuplicateFunctionId::validate(self, validate);
        DuplicateEventId::validate(self, validate);
        NonCamelCaseService::validate(self, validate);

        self.name.validate(true, validate);

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

    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    pub fn doc(&self) -> &[DocString] {
        &self.doc
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn uuid_comment(&self) -> &[Comment] {
        &self.uuid_comment
    }

    pub fn uuid(&self) -> &LitUuid {
        &self.uuid
    }

    pub fn version_comment(&self) -> &[Comment] {
        &self.ver_comment
    }

    pub fn version(&self) -> &LitInt {
        &self.ver
    }

    pub fn items(&self) -> &[ServiceItem] {
        &self.items
    }

    pub fn function_fallback(&self) -> Option<&FunctionFallback> {
        self.fn_fallback.as_ref()
    }

    pub fn event_fallback(&self) -> Option<&EventFallback> {
        self.ev_fallback.as_ref()
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
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

    pub fn doc(&self) -> &[DocString] {
        match self {
            Self::Function(i) => i.doc(),
            Self::Event(i) => i.doc(),
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
    comment: Vec<Comment>,
    doc: Vec<DocString>,
    name: Ident,
    id: LitInt,
    args: Option<FunctionPart>,
    ok: Option<FunctionPart>,
    err: Option<FunctionPart>,
}

impl FunctionDef {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::fn_def);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip @.

        let pair = pairs.next().unwrap();
        let id = LitInt::parse(pair);

        let mut args = None;
        let mut ok = None;
        let mut err = None;
        for pair in pairs {
            match pair.as_rule() {
                Rule::fn_args => args = Some(FunctionPart::parse(pair)),
                Rule::fn_ok => ok = Some(FunctionPart::parse(pair)),
                Rule::fn_err => err = Some(FunctionPart::parse(pair)),
                Rule::type_name_or_inline => ok = Some(FunctionPart::parse(pair)),

                Rule::tok_cur_open | Rule::tok_cur_close | Rule::tok_eq | Rule::tok_term => {}
                _ => unreachable!(),
            }
        }

        Self {
            span,
            comment: prelude.take_comment(),
            doc: prelude.take_doc(),
            name,
            id,
            args,
            ok,
            err,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        BrokenDocLink::validate(&self.doc, validate);
        NonSnakeCaseFunction::validate(self, validate);
        InvalidFunctionId::validate(self, validate);

        self.name.validate(true, validate);

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

    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    pub fn doc(&self) -> &[DocString] {
        &self.doc
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn id(&self) -> &LitInt {
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
    comment: Vec<Comment>,
    part_type: TypeNameOrInline,
}

impl FunctionPart {
    fn parse(pair: Pair<Rule>) -> Self {
        let span = Span::from_pair(&pair);

        let (comment, part_type) = match pair.as_rule() {
            Rule::fn_args | Rule::fn_ok | Rule::fn_err => {
                let mut pairs = pair.into_inner();
                let mut prelude = Prelude::regular(&mut pairs);

                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip =.

                let pair = pairs.next().unwrap();
                (prelude.take_comment(), TypeNameOrInline::parse(pair))
            }

            Rule::type_name_or_inline => (Vec::new(), TypeNameOrInline::parse(pair)),
            _ => unreachable!(),
        };

        Self {
            span,
            comment,
            part_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        self.part_type.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    pub fn part_type(&self) -> &TypeNameOrInline {
        &self.part_type
    }
}

#[derive(Debug, Clone)]
pub struct EventDef {
    span: Span,
    comment: Vec<Comment>,
    doc: Vec<DocString>,
    name: Ident,
    id: LitInt,
    event_type: Option<TypeNameOrInline>,
}

impl EventDef {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::event_def);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip @.

        let pair = pairs.next().unwrap();
        let id = LitInt::parse(pair);

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
            comment: prelude.take_comment(),
            doc: prelude.take_doc(),
            name,
            id,
            event_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        BrokenDocLink::validate(&self.doc, validate);
        NonSnakeCaseEvent::validate(self, validate);
        InvalidEventId::validate(self, validate);

        self.name.validate(true, validate);

        if let Some(ref event_type) = self.event_type {
            event_type.validate(validate);
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    pub fn doc(&self) -> &[DocString] {
        &self.doc
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn id(&self) -> &LitInt {
        &self.id
    }

    pub fn event_type(&self) -> Option<&TypeNameOrInline> {
        self.event_type.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionFallback {
    span: Span,
    comment: Vec<Comment>,
    doc: Vec<DocString>,
    name: Ident,
}

impl FunctionFallback {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::fn_fallback);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        Self {
            span,
            comment: prelude.take_comment(),
            doc: prelude.take_doc(),
            name,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        self.name.validate(true, validate);

        BrokenDocLink::validate(&self.doc, validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    pub fn doc(&self) -> &[DocString] {
        &self.doc
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct EventFallback {
    span: Span,
    comment: Vec<Comment>,
    doc: Vec<DocString>,
    name: Ident,
}

impl EventFallback {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::event_fallback);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        Self {
            span,
            comment: prelude.take_comment(),
            doc: prelude.take_doc(),
            name,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        self.name.validate(true, validate);

        BrokenDocLink::validate(&self.doc, validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    pub fn doc(&self) -> &[DocString] {
        &self.doc
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}
