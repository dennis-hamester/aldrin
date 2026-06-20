use crate::Span;
use crate::error::{Expected, ParseError};
use crate::lexer::Token;
use chumsky::Parser;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::primitive::just;

macro_rules! def_punct {
    ($ty:ident, $punct:literal, $label:expr) => {
        #[derive(Debug, Copy, Clone)]
        pub(crate) struct $ty;

        impl $ty {
            pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>> + Clone
            where
                I: ValueInput<'a, Token = Token<'a>, Span = Span>,
            {
                just(Token::Punct($punct)).to(Self).labelled($label)
            }
        }
    };
}

def_punct!(PunctAngClose, ">", Expected::PUNCT_ANG_CLOSE);
def_punct!(PunctAngOpen, "<", Expected::PUNCT_ANG_OPEN);
def_punct!(PunctArrow, "->", Expected::PUNCT_ARROW);
def_punct!(PunctAt, "@", Expected::PUNCT_AT);
def_punct!(PunctComma, ",", Expected::PUNCT_COMMA);
def_punct!(PunctCurClose, "}", Expected::PUNCT_CUR_CLOSE);
def_punct!(PunctCurOpen, "{", Expected::PUNCT_CUR_OPEN);
def_punct!(PunctEq, "=", Expected::PUNCT_EQ);
def_punct!(PunctExclamation, "!", Expected::PUNCT_EXCLAMATION);
def_punct!(PunctHash, "#", Expected::PUNCT_HASH);
def_punct!(PunctParClose, ")", Expected::PUNCT_PAR_CLOSE);
def_punct!(PunctParOpen, "(", Expected::PUNCT_PAR_OPEN);
def_punct!(PunctScope, "::", Expected::PUNCT_SCOPE);
def_punct!(PunctSemicolon, ";", Expected::PUNCT_SEMICOLON);
def_punct!(PunctSquClose, "]", Expected::PUNCT_SQU_CLOSE);
def_punct!(PunctSquOpen, "[", Expected::PUNCT_SQU_OPEN);
