use super::{ArrayLen, KeyTypeName, NamedRef};
use crate::error::{ExpectedTypeFoundConst, ExpectedTypeFoundService, TypeNotFound};
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

        Self { span, kind }
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
    Lifetime,
    Unit,
    Result(Box<TypeName>, Box<TypeName>),
    Array(Box<TypeName>, ArrayLen),
    Ref(NamedRef),
}

impl TypeNameKind {
    fn parse(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::kw_bool => Self::Bool,
            Rule::kw_u8 => Self::U8,
            Rule::kw_i8 => Self::I8,
            Rule::kw_u16 => Self::U16,
            Rule::kw_i16 => Self::I16,
            Rule::kw_u32 => Self::U32,
            Rule::kw_i32 => Self::I32,
            Rule::kw_u64 => Self::U64,
            Rule::kw_i64 => Self::I64,
            Rule::kw_f32 => Self::F32,
            Rule::kw_f64 => Self::F64,
            Rule::kw_string => Self::String,
            Rule::kw_uuid => Self::Uuid,
            Rule::kw_object_id => Self::ObjectId,
            Rule::kw_service_id => Self::ServiceId,
            Rule::kw_value => Self::Value,
            Rule::kw_lifetime => Self::Lifetime,
            Rule::kw_unit => Self::Unit,

            Rule::option_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();

                Self::Option(Box::new(TypeName::parse(pair)))
            }

            Rule::box_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();

                Self::Box(Box::new(TypeName::parse(pair)))
            }

            Rule::vec_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();

                Self::Vec(Box::new(TypeName::parse(pair)))
            }

            Rule::kw_bytes => Self::Bytes,

            Rule::map_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let key_pair = pairs.next().unwrap();
                pairs.next().unwrap(); // Skip ->.
                let type_pair = pairs.next().unwrap();

                Self::Map(
                    KeyTypeName::parse(key_pair),
                    Box::new(TypeName::parse(type_pair)),
                )
            }

            Rule::set_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();

                Self::Set(KeyTypeName::parse(pair))
            }

            Rule::sender_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();

                Self::Sender(Box::new(TypeName::parse(pair)))
            }

            Rule::receiver_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();

                Self::Receiver(Box::new(TypeName::parse(pair)))
            }

            Rule::result_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let ok_pair = pairs.next().unwrap();
                pairs.next().unwrap(); // Skip ,.
                let err_pair = pairs.next().unwrap();

                Self::Result(
                    Box::new(TypeName::parse(ok_pair)),
                    Box::new(TypeName::parse(err_pair)),
                )
            }

            Rule::array_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip [.
                let type_pair = pairs.next().unwrap();
                pairs.next().unwrap(); // Skip ;.
                let len_pair = pairs.next().unwrap();

                Self::Array(
                    Box::new(TypeName::parse(type_pair)),
                    ArrayLen::parse(len_pair),
                )
            }

            Rule::named_ref => Self::Ref(NamedRef::parse(pair)),

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

            Self::Result(ok, err) => {
                ok.validate(validate);
                err.validate(validate);
            }

            Self::Array(ty, len) => {
                ty.validate(validate);
                len.validate(validate);
            }

            Self::Ref(ty) => {
                TypeNotFound::validate(ty, validate);
                ExpectedTypeFoundService::validate(ty, validate);
                ExpectedTypeFoundConst::validate(ty, validate);
                ty.validate(validate);
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
            | Self::Set(_)
            | Self::Lifetime
            | Self::Unit => {}
        }
    }
}
