use super::grammar::Rule;
use super::{InlineEnum, InlineStruct};
use crate::error::Error;
use pest::iterators::Pair;

#[derive(Debug)]
pub(crate) enum Type {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    String,
    Uuid,
    Value,
    Vec(Box<Type>),
    Bytes,
    Map(MapKeyType, Box<Type>),
    Set(MapKeyType),
    External(String, String),
    Internal(String),
}

impl Type {
    pub fn from_type_name(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::bool_type => Ok(Type::Bool),

            Rule::int_type => match pair.as_str() {
                "u8" => Ok(Type::U8),
                "u16" => Ok(Type::U16),
                "u32" => Ok(Type::U32),
                "u64" => Ok(Type::U64),
                "i8" => Ok(Type::I8),
                "i16" => Ok(Type::I16),
                "i32" => Ok(Type::I32),
                "i64" => Ok(Type::I64),
                _ => unreachable!(),
            },

            Rule::float_type => match pair.as_str() {
                "f32" => Ok(Type::F32),
                "f64" => Ok(Type::F64),
                _ => unreachable!(),
            },

            Rule::string_type => Ok(Type::String),
            Rule::uuid_type => Ok(Type::Uuid),
            Rule::value_type => Ok(Type::Value),

            Rule::vec_type => {
                let elem_type = Self::from_type_name(pair.into_inner().next().unwrap())?;
                Ok(Type::Vec(Box::new(elem_type)))
            }

            Rule::bytes_type => Ok(Type::Bytes),

            Rule::map_type => {
                let mut pairs = pair.into_inner();
                let key_type = MapKeyType::from_map_key_type(pairs.next().unwrap())?;
                let value_type = Self::from_type_name(pairs.next().unwrap())?;
                Ok(Type::Map(key_type, Box::new(value_type)))
            }

            Rule::set_type => {
                let elem_type = MapKeyType::from_map_key_type(pair.into_inner().next().unwrap())?;
                Ok(Type::Set(elem_type))
            }

            Rule::extern_type_name => {
                let mut pairs = pair.into_inner();
                let module = pairs.next().unwrap().as_str().to_owned();
                let ident = pairs.next().unwrap().as_str().to_owned();
                Ok(Type::External(module, ident))
            }

            Rule::ident => {
                let ident = pair.as_str().to_owned();
                Ok(Type::Internal(ident))
            }

            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum MapKeyType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    String,
    Uuid,
}

impl MapKeyType {
    pub fn from_map_key_type(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::int_type => match pair.as_str() {
                "u8" => Ok(MapKeyType::U8),
                "u16" => Ok(MapKeyType::U16),
                "u32" => Ok(MapKeyType::U32),
                "u64" => Ok(MapKeyType::U64),
                "i8" => Ok(MapKeyType::I8),
                "i16" => Ok(MapKeyType::I16),
                "i32" => Ok(MapKeyType::I32),
                "i64" => Ok(MapKeyType::I64),
                _ => unreachable!(),
            },

            Rule::string_type => Ok(MapKeyType::String),
            Rule::uuid_type => Ok(MapKeyType::Uuid),

            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum TypeOrInline {
    Type(Type),
    Struct(InlineStruct),
    Enum(InlineEnum),
}

impl TypeOrInline {
    pub fn from_type_name_or_inline(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::struct_inline => Ok(TypeOrInline::Struct(InlineStruct::from_struct_inline(
                pair,
            )?)),
            Rule::enum_inline => Ok(TypeOrInline::Enum(InlineEnum::from_enum_inline(pair)?)),
            _ => Ok(TypeOrInline::Type(Type::from_type_name(pair)?)),
        }
    }
}
