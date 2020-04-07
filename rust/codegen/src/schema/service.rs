use super::grammar::Rule;
use super::{Ident, TypeOrInline};
use crate::error::Error;
use pest::iterators::Pair;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct Service {
    pub name: Ident,
    pub uuid: Uuid,
    pub ver: u32,
    pub elems: Vec<ServiceElement>,
}

impl Service {
    pub fn from_service_def(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::service_def);
        let mut pairs = pair.into_inner();
        let name = Ident::from_string(pairs.next().unwrap().as_str())?;
        let uuid = pairs.next().unwrap().as_str().parse().unwrap();
        let ver = pairs.next().unwrap().as_str().parse().unwrap();

        let mut this = Service {
            name,
            uuid,
            ver,
            elems: Vec::new(),
        };

        for pair in pairs {
            match pair.as_rule() {
                Rule::fn_def => this.fn_def(pair)?,
                Rule::event_def => this.event_def(pair)?,
                _ => unreachable!(),
            }
        }

        Ok(this)
    }

    fn fn_def(&mut self, pair: Pair<Rule>) -> Result<(), Error> {
        assert_eq!(pair.as_rule(), Rule::fn_def);
        self.elems
            .push(ServiceElement::Function(Function::from_fn_def(pair)?));
        Ok(())
    }

    fn event_def(&mut self, pair: Pair<Rule>) -> Result<(), Error> {
        assert_eq!(pair.as_rule(), Rule::event_def);
        self.elems
            .push(ServiceElement::Event(Event::from_event_def(pair)?));
        Ok(())
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub(crate) enum ServiceElement {
    Function(Function),
    Event(Event),
}

#[derive(Debug)]
pub(crate) struct Function {
    pub name: Ident,
    pub id: u32,
    pub args: Option<TypeOrInline>,
    pub args_required: bool,
    pub ok: Option<TypeOrInline>,
    pub ok_required: bool,
    pub err: Option<TypeOrInline>,
    pub err_required: bool,
}

impl Function {
    fn from_fn_def(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::fn_def);
        let mut pairs = pair.into_inner();

        let pair = pairs.next().unwrap();
        let name = Ident::from_string(pair.as_str())?;
        let id = pairs.next().unwrap().as_str().parse().unwrap();

        let mut args = None;
        let mut args_required = true;
        let mut ok = None;
        let mut ok_required = true;
        let mut err = None;
        let mut err_required = true;
        for _ in 0..3 {
            if let Some(pair) = pairs.next() {
                match pair.as_rule() {
                    Rule::fn_args => {
                        let mut pairs = pair.into_inner();
                        let mut pair = pairs.next().unwrap();
                        if pair.as_rule() == Rule::optional_mark {
                            args_required = false;
                            pair = pairs.next().unwrap();
                        }
                        args = Some(TypeOrInline::from_type_name_or_inline(pair)?);
                    }
                    Rule::fn_ok => {
                        let mut pairs = pair.into_inner();
                        let mut pair = pairs.next().unwrap();
                        if pair.as_rule() == Rule::optional_mark {
                            ok_required = false;
                            pair = pairs.next().unwrap();
                        }
                        ok = Some(TypeOrInline::from_type_name_or_inline(pair)?);
                    }
                    Rule::fn_err => {
                        let mut pairs = pair.into_inner();
                        let mut pair = pairs.next().unwrap();
                        if pair.as_rule() == Rule::optional_mark {
                            err_required = false;
                            pair = pairs.next().unwrap();
                        }
                        err = Some(TypeOrInline::from_type_name_or_inline(pair)?);
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(Function {
            name,
            id,
            args,
            args_required,
            ok,
            ok_required,
            err,
            err_required,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Event {
    pub name: Ident,
    pub id: u32,
    pub event_type: Option<TypeOrInline>,
    pub required: bool,
}

impl Event {
    fn from_event_def(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::event_def);
        let mut pairs = pair.into_inner();
        let name = Ident::from_string(pairs.next().unwrap().as_str())?;
        let id = pairs.next().unwrap().as_str().parse().unwrap();

        let mut pair = pairs.next();

        let required = if pair.as_ref().map(Pair::as_rule) == Some(Rule::optional_mark) {
            pair = pairs.next();
            false
        } else {
            true
        };

        let event_type = pair
            .map(TypeOrInline::from_type_name_or_inline)
            .transpose()?;

        Ok(Event {
            name,
            id,
            event_type,
            required,
        })
    }
}
