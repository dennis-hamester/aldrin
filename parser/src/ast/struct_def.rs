use super::{Attribute, Ident, LitPosInt, TypeName};
use crate::error::{
    DuplicateStructField, DuplicateStructFieldId, InvalidStructFieldId, RecursiveStruct,
};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::warning::{NonCamelCaseStruct, NonSnakeCaseStructField};
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct StructDef {
    span: Span,
    attrs: Vec<Attribute>,
    name: Ident,
    fields: Vec<StructField>,
    fallback: Option<Ident>,
}

impl StructDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_def);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        let mut attrs = Vec::new();
        for pair in &mut pairs {
            match pair.as_rule() {
                Rule::attribute => attrs.push(Attribute::parse(pair)),
                Rule::kw_struct => break,
                _ => unreachable!(),
            }
        }

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip {.

        let mut fields = Vec::new();
        let mut fallback = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::struct_field => fields.push(StructField::parse(pair)),

                Rule::struct_fallback => {
                    let mut pairs = pair.into_inner();
                    let pair = pairs.next().unwrap();
                    fallback = Some(Ident::parse(pair));
                }

                Rule::tok_cur_close => break,
                _ => unreachable!(),
            }
        }

        Self {
            span,
            attrs,
            name,
            fields,
            fallback,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        DuplicateStructField::validate(
            &self.fields,
            self.fallback.as_ref(),
            self.name.span(),
            Some(&self.name),
            validate,
        );

        DuplicateStructFieldId::validate(
            &self.fields,
            self.name.span(),
            Some(&self.name),
            validate,
        );

        NonCamelCaseStruct::validate(self, validate);
        RecursiveStruct::validate(self, validate);

        self.name.validate(validate);

        for field in &self.fields {
            field.validate(validate);
        }

        if let Some(ref fallback) = self.fallback {
            NonSnakeCaseStructField::validate(fallback, validate);
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

    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }

    pub fn fallback(&self) -> Option<&Ident> {
        self.fallback.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct InlineStruct {
    span: Span,
    kw_span: Span,
    fields: Vec<StructField>,
    fallback: Option<Ident>,
}

impl InlineStruct {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_inline);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        let pair = pairs.next().unwrap();
        let kw_span = Span::from_pair(&pair);

        pairs.next().unwrap(); // Skip {.

        let mut fields = Vec::new();
        let mut fallback = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::struct_field => fields.push(StructField::parse(pair)),

                Rule::struct_fallback => {
                    let mut pairs = pair.into_inner();
                    let pair = pairs.next().unwrap();
                    fallback = Some(Ident::parse(pair));
                }

                Rule::tok_cur_close => break,
                _ => unreachable!(),
            }
        }

        Self {
            span,
            kw_span,
            fields,
            fallback,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        DuplicateStructField::validate(
            &self.fields,
            self.fallback.as_ref(),
            self.kw_span,
            None,
            validate,
        );

        DuplicateStructFieldId::validate(&self.fields, self.kw_span, None, validate);

        for field in &self.fields {
            field.validate(validate);
        }

        if let Some(ref fallback) = self.fallback {
            NonSnakeCaseStructField::validate(fallback, validate);
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn keyword_span(&self) -> Span {
        self.kw_span
    }

    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }

    pub fn fallback(&self) -> Option<&Ident> {
        self.fallback.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct StructField {
    span: Span,
    req: bool,
    name: Ident,
    id: LitPosInt,
    field_type: TypeName,
}

impl StructField {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_field);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();

        let pair = pairs.next().unwrap();
        let req;
        let name;
        match pair.as_rule() {
            Rule::kw_required => {
                req = true;
                let pair = pairs.next().unwrap();
                name = Ident::parse(pair);
            }
            Rule::ident => {
                req = false;
                name = Ident::parse(pair);
            }
            _ => unreachable!(),
        }

        pairs.next().unwrap(); // Skip @.

        let pair = pairs.next().unwrap();
        let id = LitPosInt::parse(pair);

        pairs.next().unwrap(); // Skip =.

        let pair = pairs.next().unwrap();
        let field_type = TypeName::parse(pair);

        Self {
            span,
            req,
            name,
            id,
            field_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        InvalidStructFieldId::validate(self, validate);
        NonSnakeCaseStructField::validate(&self.name, validate);

        self.name.validate(validate);
        self.field_type.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn required(&self) -> bool {
        self.req
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn id(&self) -> &LitPosInt {
        &self.id
    }

    pub fn field_type(&self) -> &TypeName {
        &self.field_type
    }
}
