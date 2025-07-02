use super::{ArrayLen, NamedRef};
use crate::error::{ExpectedTypeFoundConst, ExpectedTypeFoundService, TypeNotFound};
use crate::grammar::Rule;
use crate::validate::Validate;
use crate::Span;
use pest::iterators::Pair;
use std::fmt;

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
    Map(Box<TypeName>, Box<TypeName>),
    Set(Box<TypeName>),
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
                    Box::new(TypeName::parse(key_pair)),
                    Box::new(TypeName::parse(type_pair)),
                )
            }

            Rule::set_type => {
                let mut pairs = pair.into_inner();
                pairs.next().unwrap(); // Skip keyword.
                pairs.next().unwrap(); // Skip <.
                let pair = pairs.next().unwrap();

                Self::Set(Box::new(TypeName::parse(pair)))
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
            | Self::Sender(ty)
            | Self::Receiver(ty) => ty.validate(validate),

            Self::Map(k, t) => {
                k.validate(validate);
                t.validate(validate);
            }

            Self::Set(ty) => {
                ty.validate(validate);
            }

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
            | Self::Lifetime
            | Self::Unit => {}
        }
    }
}

impl fmt::Display for TypeNameKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::U8 => write!(f, "u8"),
            Self::I8 => write!(f, "i8"),
            Self::U16 => write!(f, "u16"),
            Self::I16 => write!(f, "i16"),
            Self::U32 => write!(f, "u32"),
            Self::I32 => write!(f, "i32"),
            Self::U64 => write!(f, "u64"),
            Self::I64 => write!(f, "i64"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::String => write!(f, "string"),
            Self::Uuid => write!(f, "uuid"),
            Self::ObjectId => write!(f, "object_id"),
            Self::ServiceId => write!(f, "service_id"),
            Self::Value => write!(f, "value"),
            Self::Option(ty) => write!(f, "option<{}>", ty.kind()),
            Self::Box(ty) => write!(f, "box<{}>", ty.kind()),
            Self::Vec(ty) => write!(f, "vec<{}>", ty.kind()),
            Self::Bytes => write!(f, "bytes"),
            Self::Map(k, t) => write!(f, "map<{} -> {}>", k.kind(), t.kind()),
            Self::Set(ty) => write!(f, "set<{}>", ty.kind()),
            Self::Sender(ty) => write!(f, "sender<{}>", ty.kind()),
            Self::Receiver(ty) => write!(f, "receiver<{}>", ty.kind()),
            Self::Lifetime => write!(f, "lifetime"),
            Self::Unit => write!(f, "unit"),
            Self::Result(ok, err) => write!(f, "result<{}, {}>", ok.kind(), err.kind()),
            Self::Array(ty, len) => write!(f, "[{}; {}]", ty.kind(), len.value()),
            Self::Ref(ty) => ty.kind().fmt(f),
        }
    }
}
