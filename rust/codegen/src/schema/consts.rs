use super::grammar::Rule;
use super::Ident;
use crate::error::Error;
use pest::iterators::Pair;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) enum Const {
    U8(Ident, u8),
    I8(Ident, i8),
    U16(Ident, u16),
    I16(Ident, i16),
    U32(Ident, u32),
    I32(Ident, i32),
    U64(Ident, u64),
    I64(Ident, i64),
    String(Ident, String),
    Uuid(Ident, Uuid),
}

impl Const {
    pub fn from_const_def(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::const_def);
        let mut pairs = pair.into_inner();
        let name = Ident::from_string(pairs.next().unwrap().as_str())?;

        let value = pairs.next().unwrap();
        let value_rule = value.as_rule();
        let value = value.into_inner().next().unwrap();
        match value_rule {
            Rule::const_u8 => Ok(Const::U8(name, value.as_str().parse()?)),
            Rule::const_i8 => Ok(Const::I8(name, value.as_str().parse()?)),
            Rule::const_u16 => Ok(Const::U16(name, value.as_str().parse()?)),
            Rule::const_i16 => Ok(Const::I16(name, value.as_str().parse()?)),
            Rule::const_u32 => Ok(Const::U32(name, value.as_str().parse()?)),
            Rule::const_i32 => Ok(Const::I32(name, value.as_str().parse()?)),
            Rule::const_u64 => Ok(Const::U64(name, value.as_str().parse()?)),
            Rule::const_i64 => Ok(Const::I64(name, value.as_str().parse()?)),
            Rule::const_string => {
                let value = value.as_str();
                Ok(Const::String(name, value[1..(value.len() - 1)].to_owned()))
            }
            Rule::const_uuid => Ok(Const::Uuid(name, value.as_str().parse().unwrap())),
            _ => unreachable!(),
        }
    }
}
