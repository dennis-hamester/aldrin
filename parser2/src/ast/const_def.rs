use super::{
    Comment, DocComment, Ident, KwConst, KwI8, KwI16, KwI32, KwI64, KwString, KwU8, KwU16, KwU32,
    KwU64, KwUuid, LitInt, LitString, LitUuid, Prelude, PunctEq, PunctParClose, PunctParOpen,
    PunctSemicolon,
};
use crate::error::ParseError;
use crate::lexer::Token;
use crate::{Span, Spanned};
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::{choice, group};

#[derive(Debug, Clone)]
pub struct Const {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub name: Ident,
    pub ty: ConstType,
    pub value: ConstValue,
}

impl Const {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Prelude::comment_parser(),
            KwConst::parser(),
            Ident::parser(),
            PunctEq::parser(),
            ConstType::parser(),
            PunctParOpen::parser(),
            ConstValue::parser(),
            PunctParClose::parser(),
            PunctSemicolon::parser(),
        ))
        .map(|(prelude, _, name, _, ty, _, value, _, _)| Self {
            comments: prelude.comments,
            doc_comments: prelude.doc_comments,
            name,
            ty,
            value,
        })
        .boxed()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConstType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    String,
    Uuid,
}

impl ConstType {
    fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        choice((
            KwU8::parser().to(Self::U8),
            KwI8::parser().to(Self::I8),
            KwU16::parser().to(Self::U16),
            KwI16::parser().to(Self::I16),
            KwU32::parser().to(Self::U32),
            KwI32::parser().to(Self::I32),
            KwU64::parser().to(Self::U64),
            KwI64::parser().to(Self::I64),
            KwString::parser().to(Self::String),
            KwUuid::parser().to(Self::Uuid),
        ))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ConstValue {
    Int(LitInt),
    String(LitString),
    Uuid(LitUuid),
}

impl ConstValue {
    fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        choice((
            LitInt::parser().map(Self::Int),
            LitString::parser().map(Self::String),
            LitUuid::parser().map(Self::Uuid),
        ))
    }

    pub fn is_int(self) -> bool {
        matches!(self, Self::Int(_))
    }

    pub fn as_int(self) -> Option<LitInt> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Int(val) => Some(val),
            _ => None,
        }
    }

    pub fn is_string(self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn as_string(self) -> Option<LitString> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::String(val) => Some(val),
            _ => None,
        }
    }

    pub fn is_uuid(self) -> bool {
        matches!(self, Self::Uuid(_))
    }

    pub fn as_uuid(self) -> Option<LitUuid> {
        #[expect(clippy::wildcard_enum_match_arm)]
        match self {
            Self::Uuid(val) => Some(val),
            _ => None,
        }
    }
}

impl Spanned for ConstValue {
    fn span(&self) -> Span {
        match self {
            Self::Int(val) => val.span(),
            Self::String(val) => val.span(),
            Self::Uuid(val) => val.span(),
        }
    }
}
