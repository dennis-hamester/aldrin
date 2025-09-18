use super::{Attribute, Ident, LitInt, Prelude, TypeName};
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
    doc: Option<String>,
    attrs: Vec<Attribute>,
    name: Ident,
    fields: Vec<StructField>,
    fallback: Option<StructFallback>,
}

impl StructDef {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_def);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::new(&mut pairs, false);

        pairs.next().unwrap(); // Skip keyword.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        pairs.next().unwrap(); // Skip {.

        let mut fields = Vec::new();
        let mut fallback = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::struct_field => fields.push(StructField::parse(pair)),
                Rule::struct_fallback => fallback = Some(StructFallback::parse(pair)),
                Rule::tok_cur_close => break,
                _ => unreachable!(),
            }
        }

        Self {
            span,
            doc: prelude.take_doc().into(),
            attrs: prelude.take_attrs(),
            name,
            fields,
            fallback,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        DuplicateStructField::validate(
            &self.fields,
            self.fallback.as_ref(),
            Some(&self.name),
            validate,
        );

        DuplicateStructFieldId::validate(&self.fields, Some(&self.name), validate);
        NonCamelCaseStruct::validate(self, validate);
        RecursiveStruct::validate(self, validate);

        self.name.validate(true, validate);

        for field in &self.fields {
            field.validate(validate);
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

    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }

    pub fn fallback(&self) -> Option<&StructFallback> {
        self.fallback.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct InlineStruct {
    span: Span,
    kw_span: Span,
    doc: Option<String>,
    attrs: Vec<Attribute>,
    fields: Vec<StructField>,
    fallback: Option<StructFallback>,
}

impl InlineStruct {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_inline);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();

        let pair = pairs.next().unwrap();
        let kw_span = Span::from_pair(&pair);

        pairs.next().unwrap(); // Skip {.

        let mut prelude = Prelude::new(&mut pairs, true);
        let mut fields = Vec::new();
        let mut fallback = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::struct_field => fields.push(StructField::parse(pair)),
                Rule::struct_fallback => fallback = Some(StructFallback::parse(pair)),
                Rule::tok_cur_close => break,
                _ => unreachable!(),
            }
        }

        Self {
            span,
            kw_span,
            doc: prelude.take_inline_doc().into(),
            attrs: prelude.take_attrs_inline(),
            fields,
            fallback,
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        DuplicateStructField::validate(&self.fields, self.fallback.as_ref(), None, validate);
        DuplicateStructFieldId::validate(&self.fields, None, validate);

        for field in &self.fields {
            field.validate(validate);
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

    pub fn attributes(&self) -> &[Attribute] {
        &self.attrs
    }

    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }

    pub fn fallback(&self) -> Option<&StructFallback> {
        self.fallback.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct StructField {
    span: Span,
    doc: Option<String>,
    req: bool,
    name: Ident,
    id: LitInt,
    field_type: TypeName,
}

impl StructField {
    fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_field);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::new(&mut pairs, false);

        let (req, name) = match pairs.next().map(|pair| (pair.as_rule(), pair)).unwrap() {
            (Rule::kw_required, _) => (true, Ident::parse(pairs.next().unwrap())),
            (Rule::ident, pair) => (false, Ident::parse(pair)),
            _ => unreachable!(),
        };

        pairs.next().unwrap(); // Skip @.

        let pair = pairs.next().unwrap();
        let id = LitInt::parse(pair);

        pairs.next().unwrap(); // Skip =.

        let pair = pairs.next().unwrap();
        let field_type = TypeName::parse(pair);

        Self {
            span,
            doc: prelude.take_doc().into(),
            req,
            name,
            id,
            field_type,
        }
    }

    fn validate(&self, validate: &mut Validate) {
        InvalidStructFieldId::validate(self, validate);
        NonSnakeCaseStructField::validate(&self.name, validate);

        self.name.validate(true, validate);
        self.field_type.validate(false, validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn required(&self) -> bool {
        self.req
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn id(&self) -> &LitInt {
        &self.id
    }

    pub fn field_type(&self) -> &TypeName {
        &self.field_type
    }
}

#[derive(Debug, Clone)]
pub struct StructFallback {
    span: Span,
    doc: Option<String>,
    name: Ident,
}

impl StructFallback {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::struct_fallback);

        let span = Span::from_pair(&pair);
        let mut pairs = pair.into_inner();
        let mut prelude = Prelude::new(&mut pairs, false);

        Self {
            span,
            doc: prelude.take_doc().into(),
            name: Ident::parse(pairs.next().unwrap()),
        }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        self.name.validate(true, validate);
        NonSnakeCaseStructField::validate(&self.name, validate);
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
