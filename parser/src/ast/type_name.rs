use super::{Ident, KeyTypeName, SchemaName};
use crate::error::{ExternTypeNotFound, MissingImport, TypeNotFound};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct TypeName {
    span: Span,
    kind: TypeNameKind,
}

impl TypeName {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::type_name);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        let kind = TypeNameKind::parse(pair);

        TypeName { span, kind }
    }

    pub(crate) fn validate(&self, validate: &mut Validate) {
        self.kind.validate(validate);
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn kind(&self) -> &TypeNameKind {
        &self.kind
    }
}

#[derive(Debug, Clone)]
pub enum TypeNameKind {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    String,
    Uuid,
    ObjectId,
    ServiceId,
    Value,
    Option(Box<TypeName>),
    Box(Box<TypeName>),
    Vec(Box<TypeName>),
    Bytes,
    Map(KeyTypeName, Box<TypeName>),
    Set(KeyTypeName),
    Sender(Box<TypeName>),
    Receiver(Box<TypeName>),
    Extern(SchemaName, Ident),
    Intern(Ident),
}

impl TypeNameKind {
    fn parse(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::kw_bool => TypeNameKind::Bool,
            Rule::kw_u8 => TypeNameKind::U8,
            Rule::kw_i8 => TypeNameKind::I8,
            Rule::kw_u16 => TypeNameKind::U16,
            Rule::kw_i16 => TypeNameKind::I16,
            Rule::kw_u32 => TypeNameKind::U32,
            Rule::kw_i32 => TypeNameKind::I32,
            Rule::kw_u64 => TypeNameKind::U64,
            Rule::kw_i64 => TypeNameKind::I64,
            Rule::kw_f32 => TypeNameKind::F32,
            Rule::kw_f64 => TypeNameKind::F64,
            Rule::kw_string => TypeNameKind::String,
            Rule::kw_uuid => TypeNameKind::Uuid,
            Rule::kw_object_id => TypeNameKind::ObjectId,
            Rule::kw_service_id => TypeNameKind::ServiceId,
            Rule::kw_value => TypeNameKind::Value,

            Rule::option_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();
                TypeNameKind::Option(Box::new(TypeName::parse(pair)))
            }

            Rule::box_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();
                TypeNameKind::Box(Box::new(TypeName::parse(pair)))
            }

            Rule::vec_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();
                TypeNameKind::Vec(Box::new(TypeName::parse(pair)))
            }

            Rule::kw_bytes => TypeNameKind::Bytes,

            Rule::map_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let key_pair = pairs.next().unwrap();
                pairs.next().unwrap(); // Skip ->.
                let type_pair = pairs.next().unwrap();
                TypeNameKind::Map(
                    KeyTypeName::parse(key_pair),
                    Box::new(TypeName::parse(type_pair)),
                )
            }

            Rule::set_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();
                TypeNameKind::Set(KeyTypeName::parse(pair))
            }

            Rule::sender_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();
                TypeNameKind::Sender(Box::new(TypeName::parse(pair)))
            }

            Rule::receiver_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();
                TypeNameKind::Receiver(Box::new(TypeName::parse(pair)))
            }

            Rule::external_type_name => {
                let mut pairs = pair.into_inner();
                let pair = pairs.next().unwrap();
                let schema_name = SchemaName::parse(pair);
                pairs.next().unwrap(); // Skip ::.
                let ident = Ident::parse(pairs.next().unwrap());
                TypeNameKind::Extern(schema_name, ident)
            }

            Rule::ident => TypeNameKind::Intern(Ident::parse(pair)),

            _ => unreachable!(),
        }
    }

    fn validate(&self, validate: &mut Validate) {
        match self {
            Self::Option(ty)
            | Self::Box(ty)
            | Self::Vec(ty)
            | Self::Map(_, ty)
            | Self::Sender(ty)
            | Self::Receiver(ty) => ty.validate(validate),

            Self::Extern(schema, ty) => {
                MissingImport::validate(schema, validate);
                ExternTypeNotFound::validate(schema, ty, validate);
            }

            Self::Intern(ty) => {
                TypeNotFound::validate(ty, validate);
            }

            Self::Bool
            | Self::U8
            | Self::I8
            | Self::U16
            | Self::I16
            | Self::U32
            | Self::I32
            | Self::U64
            | Self::I64
            | Self::F32
            | Self::F64
            | Self::String
            | Self::Uuid
            | Self::ObjectId
            | Self::ServiceId
            | Self::Value
            | Self::Bytes
            | Self::Set(_) => {}
        }
    }
}
