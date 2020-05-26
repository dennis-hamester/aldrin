use super::grammar::Rule;
use super::{Attributes, Ident, TypeOrInline};
use crate::error::Error;
use pest::iterators::Pair;

#[derive(Debug)]
pub(crate) struct Struct {
    pub attributes: Attributes,
    pub name: Ident,
    pub fields: Vec<StructField>,
}

impl Struct {
    pub fn from_struct_def(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::struct_def);
        let mut pairs = pair.into_inner();
        let attributes = Attributes::from_pairs(&mut pairs)?;
        let name = Ident::from_string(pairs.next().unwrap().as_str())?;
        let fields = struct_body(pairs.next().unwrap())?;
        Ok(Struct {
            attributes,
            name,
            fields,
        })
    }
}

#[derive(Debug)]
pub(crate) struct InlineStruct {
    pub fields: Vec<StructField>,
}

impl InlineStruct {
    pub fn from_struct_inline(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::struct_inline);
        let mut pairs = pair.into_inner();
        let fields = struct_body(pairs.next().unwrap())?;
        Ok(InlineStruct { fields })
    }
}

#[derive(Debug)]
pub(crate) struct StructField {
    pub name: Ident,
    pub id: u32,
    pub field_type: TypeOrInline,
    pub required: bool,
}

fn struct_body(pair: Pair<Rule>) -> Result<Vec<StructField>, Error> {
    assert_eq!(pair.as_rule(), Rule::struct_body);
    let pairs = pair.into_inner();
    let mut res = Vec::new();
    for pair in pairs {
        assert_eq!(pair.as_rule(), Rule::struct_field);
        let mut pairs = pair.into_inner();

        let mut pair = pairs.next().unwrap();
        let required = if pair.as_rule() == Rule::required_mark {
            pair = pairs.next().unwrap();
            true
        } else {
            false
        };
        let name = Ident::from_string(pair.as_str())?;

        let id = pairs.next().unwrap().as_str().parse().unwrap();
        let field_type = TypeOrInline::from_type_name_or_inline(pairs.next().unwrap())?;
        res.push(StructField {
            name,
            id,
            field_type,
            required,
        });
    }
    Ok(res)
}
