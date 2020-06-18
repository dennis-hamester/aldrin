use super::{Attribute, Ident, LitPosInt, TypeNameOrInline};
use crate::error::{DuplicateStructField, DuplicateStructFieldId, InvalidStructFieldId};
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
        for pair in pairs {
            match pair.as_rule() {
                Rule::struct_field => fields.push(StructField::parse(pair)),
                Rule::tok_cur_close => break,
                _ => unreachable!(),
            }
        }

        StructDef {
            span,
            attrs,
            name,
            fields,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        DuplicateStructField::validate(&self.fields, validate);
        DuplicateStructFieldId::validate(&self.fields, validate);

        if validate.is_main_schema() {
            NonCamelCaseStruct::validate(self, validate);
        }

        for field in &self.fields {
            field.validate(validate);
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
}

#[derive(Debug, Clone)]
pub struct StructField {
    span: Span,
    req: bool,
    name: Ident,
    id: LitPosInt,
    field_type: TypeNameOrInline,
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
        let field_type = TypeNameOrInline::parse(pair);

        StructField {
            span,
            req,
            name,
            id,
            field_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        InvalidStructFieldId::validate(&self, validate);

        if validate.is_main_schema() {
            NonSnakeCaseStructField::validate(self, validate);
        }

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

    pub fn field_type(&self) -> &TypeNameOrInline {
        &self.field_type
    }
}
