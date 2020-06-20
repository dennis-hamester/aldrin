use super::{Attribute, Ident, LitPosInt, TypeNameOrInline};
use crate::error::{DuplicateEnumVariant, DuplicateEnumVariantId, InvalidEnumVariantId};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::{NonCamelCaseEnum, NonCamelCaseEnumVariant};
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct EnumDef {
    span: Span,
    attrs: Vec<Attribute>,
    name: Ident,
    vars: Vec<EnumVariant>,
}

impl EnumDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::enum_def);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        let mut attrs = Vec::new();
        for pair in &mut pairs {
            match pair.as_rule() {
                Rule::attribute => attrs.push(Attribute::parse(pair)),
                Rule::kw_enum => break,
                _ => unreachable!(),
            }
        }

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip {.

        let mut vars = Vec::new();
        for pair in pairs {
            match pair.as_rule() {
                Rule::enum_variant => vars.push(EnumVariant::parse(pair)),
                Rule::tok_cur_close => break,
                _ => unreachable!(),
            }
        }

        EnumDef {
            span,
            attrs,
            name,
            vars,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        DuplicateEnumVariant::validate(&self.vars, self.name.span(), Some(&self.name), validate);
        DuplicateEnumVariantId::validate(&self.vars, self.name.span(), Some(&self.name), validate);

        if validate.is_main_schema() {
            NonCamelCaseEnum::validate(self, validate);
        }

        self.name.validate(validate);
        for var in &self.vars {
            var.validate(validate);
        }
    }

    pub fn span(&self) -> Span {
        self.span
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
}

#[derive(Debug, Clone)]
pub struct InlineEnum {
    span: Span,
    kw_span: Span,
    vars: Vec<EnumVariant>,
}

impl InlineEnum {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::enum_inline);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        let pair = pairs.next().unwrap();
        let kw_span = Span::from_pair(&pair);

        pairs.next().unwrap(); // Skip {.

        let mut vars = Vec::new();
        for pair in pairs {
            match pair.as_rule() {
                Rule::enum_variant => vars.push(EnumVariant::parse(pair)),
                Rule::tok_cur_close => break,
                _ => unreachable!(),
            }
        }

        InlineEnum {
            span,
            kw_span,
            vars,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        DuplicateEnumVariant::validate(&self.vars, self.kw_span, None, validate);
        DuplicateEnumVariantId::validate(&self.vars, self.kw_span, None, validate);

        for var in &self.vars {
            var.validate(validate);
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn keyword_span(&self) -> Span {
        self.kw_span
    }

    pub fn variants(&self) -> &[EnumVariant] {
        &self.vars
    }
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    span: Span,
    name: Ident,
    id: LitPosInt,
    var_type: Option<EnumVariantType>,
}

impl EnumVariant {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::enum_variant);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip @.

        let pair = pairs.next().unwrap();
        let id = LitPosInt::parse(pair);

        let pair = pairs.next().unwrap();
        let var_type = match pair.as_rule() {
            Rule::tok_eq => {
                let pair = pairs.next().unwrap();
                Some(EnumVariantType::parse(pair))
            }
            Rule::tok_term => None,
            _ => unreachable!(),
        };

        EnumVariant {
            span,
            name,
            id,
            var_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        InvalidEnumVariantId::validate(&self, validate);

        if validate.is_main_schema() {
            NonCamelCaseEnumVariant::validate(self, validate);
        }

        self.name.validate(validate);
        if let Some(ref var_type) = self.var_type {
            var_type.validate(validate);
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

    pub fn variant_type(&self) -> Option<&EnumVariantType> {
        self.var_type.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct EnumVariantType {
    span: Span,
    opt: bool,
    var_type: TypeNameOrInline,
}

impl EnumVariantType {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::enum_variant_type);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        let opt;
        let var_type;
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::kw_optional => {
                opt = true;
                let pair = pairs.next().unwrap();
                var_type = TypeNameOrInline::parse(pair);
            }
            Rule::type_name_or_inline => {
                opt = false;
                var_type = TypeNameOrInline::parse(pair);
            }
            _ => unreachable!(),
        }

        EnumVariantType {
            span,
            opt,
            var_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        self.var_type.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn optional(&self) -> bool {
        self.opt
    }

    pub fn variant_type(&self) -> &TypeNameOrInline {
        &self.var_type
    }
}
