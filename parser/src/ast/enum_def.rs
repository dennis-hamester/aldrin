use super::{Attribute, DocString, Ident, LitInt, TypeName};
use crate::error::{
    DuplicateEnumVariant, DuplicateEnumVariantId, EmptyEnum, InvalidEnumVariantId, RecursiveEnum,
};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::{NonCamelCaseEnum, NonCamelCaseEnumVariant};
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct EnumDef {
    span: Span,
    doc: Option<String>,
    attrs: Vec<Attribute>,
    name: Ident,
    vars: Vec<EnumVariant>,
    fallback: Option<EnumFallback>,
}

impl EnumDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::enum_def);

        let span = Span::from_pair(&pair);

        let mut doc = DocString::new();
        let mut pairs = pair.into_inner();

        let mut attrs = Vec::new();
        for pair in &mut pairs {
            match pair.as_rule() {
                Rule::attribute => attrs.push(Attribute::parse(pair)),
                Rule::doc_string => doc.push(pair),
                Rule::kw_enum => break,
                _ => unreachable!(),
            }
        }

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip {.

        let mut vars = Vec::new();
        let mut fallback = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::enum_variant => vars.push(EnumVariant::parse(pair)),
                Rule::enum_fallback => fallback = Some(EnumFallback::parse(pair)),
                Rule::tok_cur_close => break,
                _ => unreachable!(),
            }
        }

        Self {
            span,
            doc: doc.into(),
            attrs,
            name,
            vars,
            fallback,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        DuplicateEnumVariant::validate(
            &self.vars,
            self.fallback.as_ref(),
            Some(&self.name),
            validate,
        );

        DuplicateEnumVariantId::validate(&self.vars, Some(&self.name), validate);
        NonCamelCaseEnum::validate(self, validate);
        RecursiveEnum::validate(self, validate);

        EmptyEnum::validate(
            &self.vars,
            self.fallback.as_ref(),
            self.span,
            Some(&self.name),
            validate,
        );

        self.name.validate(true, validate);

        for var in &self.vars {
            var.validate(validate);
        }

        if let Some(ref fallback) = self.fallback {
            fallback.validate(validate);
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attrs
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn variants(&self) -> &[EnumVariant] {
        &self.vars
    }

    pub fn fallback(&self) -> Option<&EnumFallback> {
        self.fallback.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct InlineEnum {
    span: Span,
    kw_span: Span,
    doc: Option<String>,
    vars: Vec<EnumVariant>,
    fallback: Option<EnumFallback>,
}

impl InlineEnum {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::enum_inline);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        let pair = pairs.next().unwrap();
        let kw_span = Span::from_pair(&pair);

        pairs.next().unwrap(); // Skip {.

        let mut doc = DocString::new();
        let mut vars = Vec::new();
        let mut fallback = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::doc_string_inline => doc.push_inline(pair),
                Rule::enum_variant => vars.push(EnumVariant::parse(pair)),
                Rule::enum_fallback => fallback = Some(EnumFallback::parse(pair)),
                Rule::tok_cur_close => break,
                _ => unreachable!(),
            }
        }

        Self {
            span,
            kw_span,
            doc: doc.into(),
            vars,
            fallback,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        DuplicateEnumVariant::validate(&self.vars, self.fallback.as_ref(), None, validate);
        DuplicateEnumVariantId::validate(&self.vars, None, validate);

        EmptyEnum::validate(
            &self.vars,
            self.fallback.as_ref(),
            self.span,
            None,
            validate,
        );

        for var in &self.vars {
            var.validate(validate);
        }

        if let Some(ref fallback) = self.fallback {
            fallback.validate(validate);
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn keyword_span(&self) -> Span {
        self.kw_span
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn variants(&self) -> &[EnumVariant] {
        &self.vars
    }

    pub fn fallback(&self) -> Option<&EnumFallback> {
        self.fallback.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    span: Span,
    doc: Option<String>,
    name: Ident,
    id: LitInt,
    var_type: Option<TypeName>,
}

impl EnumVariant {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::enum_variant);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();

        let mut doc = DocString::new();

        let name = loop {
            let pair = pairs.next().unwrap();

            match pair.as_rule() {
                Rule::doc_string => doc.push(pair),
                Rule::ident => break Ident::parse(pair),
                _ => unreachable!(),
            }
        };

        pairs.next().unwrap(); // Skip @.

        let pair = pairs.next().unwrap();
        let id = LitInt::parse(pair);

        let pair = pairs.next().unwrap();
        let var_type = match pair.as_rule() {
            Rule::tok_eq => {
                let pair = pairs.next().unwrap();
                Some(TypeName::parse(pair))
            }
            Rule::tok_term => None,
            _ => unreachable!(),
        };

        Self {
            span,
            doc: doc.into(),
            name,
            id,
            var_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        InvalidEnumVariantId::validate(self, validate);
        NonCamelCaseEnumVariant::validate(&self.name, validate);

        self.name.validate(true, validate);

        if let Some(ref var_type) = self.var_type {
            var_type.validate(false, validate);
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn id(&self) -> &LitInt {
        &self.id
    }

    pub fn variant_type(&self) -> Option<&TypeName> {
        self.var_type.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct EnumFallback {
    span: Span,
    doc: Option<String>,
    name: Ident,
}

impl EnumFallback {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::enum_fallback);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();

        let mut doc = DocString::new();

        let name = loop {
            let pair = pairs.next().unwrap();

            match pair.as_rule() {
                Rule::doc_string => doc.push(pair),
                Rule::ident => break Ident::parse(pair),
                _ => unreachable!(),
            }
        };

        Self {
            span,
            doc: doc.into(),
            name,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        self.name.validate(true, validate);
        NonCamelCaseEnumVariant::validate(&self.name, validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}
