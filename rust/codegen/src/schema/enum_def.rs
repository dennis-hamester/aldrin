use super::grammar::Rule;
use super::{Attributes, Ident, TypeOrInline};
use crate::error::Error;
use pest::iterators::Pair;

#[derive(Debug)]
pub(crate) struct Enum {
    pub attributes: Attributes,
    pub name: Ident,
    pub variants: Vec<EnumVariant>,
}

impl Enum {
    pub fn from_enum_def(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::enum_def);
        let mut pairs = pair.into_inner();
        let attributes = Attributes::from_pairs(&mut pairs)?;
        let name = Ident::from_string(pairs.next().unwrap().as_str())?;
        let variants = enum_body(pairs.next().unwrap())?;
        Ok(Enum {
            attributes,
            name,
            variants,
        })
    }
}

#[derive(Debug)]
pub(crate) struct InlineEnum {
    pub variants: Vec<EnumVariant>,
}

impl InlineEnum {
    pub fn from_enum_inline(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::enum_inline);
        let mut pairs = pair.into_inner();
        let variants = enum_body(pairs.next().unwrap())?;
        Ok(InlineEnum { variants })
    }
}

#[derive(Debug)]
pub(crate) struct EnumVariant {
    pub name: Ident,
    pub id: u32,
    pub variant_type: Option<TypeOrInline>,
    pub required: bool,
}

fn enum_body(pair: Pair<Rule>) -> Result<Vec<EnumVariant>, Error> {
    assert_eq!(pair.as_rule(), Rule::enum_body);
    let pairs = pair.into_inner();
    let mut res = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::enum_variant);
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

        let variant_type = pair
            .map(TypeOrInline::from_type_name_or_inline)
            .transpose()?;

        res.push(EnumVariant {
            name,
            id,
            variant_type,
            required,
        });
    }
    Ok(res)
}
