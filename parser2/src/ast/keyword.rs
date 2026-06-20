use crate::Span;
use crate::error::{Expected, ParseError};
use crate::lexer::Token;
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::just;

macro_rules! def_kw {
    ($ty:ident, $kw:literal $(, $label:expr)?) => {
        #[derive(Debug, Copy, Clone)]
        pub(crate) struct $ty;

        impl $ty {
            pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
            where
                I: ValueInput<'a, Token = Token<'a>, Span = Span>,
            {
                just(Token::Ident($kw)).to(Self)
                    $( .labelled($label) )?
            }
        }
    };
}

def_kw!(KwArgs, "args", Expected::KW_ARGS);
def_kw!(KwBox, "box");
def_kw!(KwConst, "const", Expected::KW_CONST);
def_kw!(KwEnum, "enum", Expected::KW_ENUM);
def_kw!(KwErr, "err", Expected::KW_ERR);
def_kw!(KwEvent, "event", Expected::KW_EVENT);
def_kw!(KwFallback, "fallback", Expected::KW_FALLBACK);
def_kw!(KwFn, "fn", Expected::KW_FN);
def_kw!(KwI16, "i16", Expected::KW_I16);
def_kw!(KwI32, "i32", Expected::KW_I32);
def_kw!(KwI64, "i64", Expected::KW_I64);
def_kw!(KwI8, "i8", Expected::KW_I8);
def_kw!(KwImport, "import", Expected::KW_IMPORT);
def_kw!(KwMap, "map");
def_kw!(KwNewtype, "newtype", Expected::KW_NEWTYPE);
def_kw!(KwOk, "ok", Expected::KW_OK);
def_kw!(KwOption, "option");
def_kw!(KwReceiver, "receiver");
def_kw!(KwRequired, "required", Expected::KW_REQUIRED);
def_kw!(KwResult, "result");
def_kw!(KwSender, "sender");
def_kw!(KwService, "service", Expected::KW_SERVICE);
def_kw!(KwSet, "set");
def_kw!(KwString, "string", Expected::KW_STRING);
def_kw!(KwStruct, "struct", Expected::KW_STRUCT);
def_kw!(KwU16, "u16", Expected::KW_U16);
def_kw!(KwU32, "u32", Expected::KW_U32);
def_kw!(KwU64, "u64", Expected::KW_U64);
def_kw!(KwU8, "u8", Expected::KW_U8);
def_kw!(KwUuid, "uuid", Expected::KW_UUID);
def_kw!(KwVec, "vec");
def_kw!(KwVersion, "version", Expected::KW_VERSION);
