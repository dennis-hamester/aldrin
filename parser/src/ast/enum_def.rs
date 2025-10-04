use super::{Attribute, Comment, DocString, Ident, LitInt, Prelude, TypeName};
use crate::error::{
    DuplicateEnumVariant, DuplicateEnumVariantId, EmptyEnum, InvalidEnumVariantId, RecursiveEnum,
};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::{BrokenDocLink, NonCamelCaseEnum, NonCamelCaseEnumVariant};
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct EnumDef {
    span: Span,
    comment: Vec<Comment>,
    doc: Vec<DocString>,
    attrs: Vec<Attribute>,
    name: Ident,
    vars: Vec<EnumVariant>,
    fallback: Option<EnumFallback>,
}

impl EnumDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::enum_def);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        pairs.next().unwrap(); // Skip keyword.

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
            comment: prelude.take_comment(),
            doc: prelude.take_doc(),
            attrs: prelude.take_attrs(),
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

        BrokenDocLink::validate(&self.doc, validate);
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

    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    pub fn doc(&self) -> &[DocString] {
        &self.doc
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
    doc: Vec<DocString>,
    attrs: Vec<Attribute>,
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

        let mut prelude = Prelude::inline(&mut pairs);
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
            kw_span,
            doc: prelude.take_inline_doc(),
            attrs: prelude.take_attrs_inline(),
            vars,
            fallback,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        BrokenDocLink::validate(&self.doc, validate);
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

    pub fn doc(&self) -> &[DocString] {
        &self.doc
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attrs
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
    comment: Vec<Comment>,
    doc: Vec<DocString>,
    name: Ident,
    id: LitInt,
    var_type: Option<TypeName>,
}

impl EnumVariant {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::enum_variant);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

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
            comment: prelude.take_comment(),
            doc: prelude.take_doc(),
            name,
            id,
            var_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        BrokenDocLink::validate(&self.doc, validate);
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

    pub fn variant_type(&self) -> Option<&TypeName> {
        self.var_type.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct EnumFallback {
    span: Span,
    comment: Vec<Comment>,
    doc: Vec<DocString>,
    name: Ident,
}

impl EnumFallback {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::enum_fallback);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::regular(&mut pairs);

        Self {
            span,
            comment: prelude.take_comment(),
            doc: prelude.take_doc(),
            name: Ident::parse(pairs.next().unwrap()),
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        BrokenDocLink::validate(&self.doc, validate);
        NonCamelCaseEnumVariant::validate(&self.name, validate);

        self.name.validate(true, validate);
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
