use crate::lexer::Token;
use crate::{DiagnosticRenderer, Parser, SchemaRef, Span};
use bitflags::bitflags;
use chumsky::DefaultExpected;
use chumsky::error::Error;
use chumsky::input::Input;
use chumsky::label::LabelError;
use chumsky::util::MaybeRef;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ParseError {
    schema: SchemaRef,
    span: Span,
    expected: Expected,
    found: Found,
}

impl ParseError {
    pub(crate) fn set_schema(&mut self, schema: SchemaRef) {
        self.schema = schema;
    }

    pub(crate) fn schema(&self) -> SchemaRef {
        self.schema
    }

    pub(crate) fn render(&self, renderer: &DiagnosticRenderer, parser: &Parser) -> String {
        if self.expected.contains(Expected::LIT_STRING) && (self.found == Found::LitStringError) {
            let mut report = renderer
                .error("unterminated string found", parser)
                .snippet(self.schema, self.span, "")
                .help("multi-line strings are not supported; use `\\n` to encode a newline");

            let schema = parser.schema(self.schema);
            let lit = schema.source_of(&self.span).unwrap();

            if lit.contains("\\\"") {
                report = report.help("`\\\"` is the escape code for `\"`");
            }

            report.render()
        } else {
            let mut title = "expected ".to_owned();
            let mut first = true;
            let mut iter = self.expected.into_iter().peekable();

            while let Some(expected) = iter.next() {
                if first {
                    first = false;
                } else if iter.peek().is_some() {
                    title.push_str(", ");
                } else {
                    title.push_str(" or ");
                }

                title.push_str(expected.as_str());
            }

            renderer
                .error(title, parser)
                .snippet(self.schema, self.span, self.found.as_label())
                .render()
        }
    }
}

impl<'a, I> Error<'a, I> for ParseError
where
    I: Input<'a, Span = Span, Token = Token<'a>>,
{
    fn merge(mut self, other: Self) -> Self {
        self.expected.insert(other.expected);
        self
    }
}

impl<'a, I> LabelError<'a, I, DefaultExpected<'a, Token<'a>>> for ParseError
where
    I: Input<'a, Span = Span, Token = Token<'a>>,
{
    fn expected_found<E: IntoIterator<Item = DefaultExpected<'a, Token<'a>>>>(
        expected: E,
        found: Option<MaybeRef<'a, Token<'a>>>,
        span: Span,
    ) -> Self {
        let expected = if expected
            .into_iter()
            .any(|expected| expected == DefaultExpected::EndOfInput)
        {
            Expected::EOI
        } else {
            Expected::empty()
        };

        Self {
            schema: SchemaRef::DUMMY,
            span,
            expected,
            found: found.map_or(Found::Eoi, Into::into),
        }
    }
}

impl<'a, I> LabelError<'a, I, Expected> for ParseError
where
    I: Input<'a, Span = Span, Token = Token<'a>>,
{
    fn expected_found<E: IntoIterator<Item = Expected>>(
        expected: E,
        found: Option<MaybeRef<'a, Token<'a>>>,
        span: Span,
    ) -> Self {
        Self {
            schema: SchemaRef::DUMMY,
            span,
            expected: expected.into_iter().collect(),
            found: found.map_or(Found::Eoi, Into::into),
        }
    }

    fn label_with(&mut self, label: Expected) {
        self.expected = label;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub(crate) struct Expected: u64 {
        const COMMENT =            0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001;
        const DOC_COMMENT =        0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010;
        const INLINE_DOC_COMMENT = 0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0100;
        const ATTRIBUTE =          0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1000;
        const INLINE_ATTRIBUTE =   0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0000;
        const TYPE =               0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010_0000;
        const KW_IMPORT =          0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0100_0000;
        const KW_CONST =           0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1000_0000;
        const KW_SERVICE =         0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0000_0000;
        const KW_STRUCT =          0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010_0000_0000;
        const KW_ENUM =            0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0100_0000_0000;
        const KW_NEWTYPE =         0b000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1000_0000_0000;
        const KW_U8 =              0b000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0000_0000_0000;
        const KW_I8 =              0b000_0000_0000_0000_0000_0000_0000_0000_0000_0010_0000_0000_0000;
        const KW_U16 =             0b000_0000_0000_0000_0000_0000_0000_0000_0000_0100_0000_0000_0000;
        const KW_I16 =             0b000_0000_0000_0000_0000_0000_0000_0000_0000_1000_0000_0000_0000;
        const KW_U32 =             0b000_0000_0000_0000_0000_0000_0000_0000_0001_0000_0000_0000_0000;
        const KW_I32 =             0b000_0000_0000_0000_0000_0000_0000_0000_0010_0000_0000_0000_0000;
        const KW_U64 =             0b000_0000_0000_0000_0000_0000_0000_0000_0100_0000_0000_0000_0000;
        const KW_I64 =             0b000_0000_0000_0000_0000_0000_0000_0000_1000_0000_0000_0000_0000;
        const KW_STRING =          0b000_0000_0000_0000_0000_0000_0000_0001_0000_0000_0000_0000_0000;
        const KW_UUID =            0b000_0000_0000_0000_0000_0000_0000_0010_0000_0000_0000_0000_0000;
        const LIT_INT =            0b000_0000_0000_0000_0000_0000_0000_0100_0000_0000_0000_0000_0000;
        const LIT_STRING =         0b000_0000_0000_0000_0000_0000_0000_1000_0000_0000_0000_0000_0000;
        const LIT_UUID =           0b000_0000_0000_0000_0000_0000_0001_0000_0000_0000_0000_0000_0000;
        const KW_ARGS =            0b000_0000_0000_0000_0000_0000_0010_0000_0000_0000_0000_0000_0000;
        const KW_OK =              0b000_0000_0000_0000_0000_0000_0100_0000_0000_0000_0000_0000_0000;
        const KW_ERR =             0b000_0000_0000_0000_0000_0000_1000_0000_0000_0000_0000_0000_0000;
        const KW_FN =              0b000_0000_0000_0000_0000_0001_0000_0000_0000_0000_0000_0000_0000;
        const KW_EVENT =           0b000_0000_0000_0000_0000_0010_0000_0000_0000_0000_0000_0000_0000;
        const KW_VERSION =         0b000_0000_0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000;
        const KW_REQUIRED =        0b000_0000_0000_0000_0000_1000_0000_0000_0000_0000_0000_0000_0000;
        const KW_FALLBACK =        0b000_0000_0000_0000_0001_0000_0000_0000_0000_0000_0000_0000_0000;
        const IDENT =              0b000_0000_0000_0000_0010_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_ANG_OPEN =     0b000_0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_ANG_CLOSE =    0b000_0000_0000_0000_1000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_CUR_OPEN =     0b000_0000_0000_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_CUR_CLOSE =    0b000_0000_0000_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_PAR_OPEN =     0b000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_PAR_CLOSE =    0b000_0000_0000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_SQU_OPEN =     0b000_0000_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_SQU_CLOSE =    0b000_0000_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_ARROW =        0b000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_AT =           0b000_0000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_COMMA =        0b000_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_EQ =           0b000_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_EXCLAMATION =  0b000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_HASH =         0b000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_SCOPE =        0b001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const PUNCT_SEMICOLON =    0b010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        const EOI =                0b100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
    }
}

impl Expected {
    fn as_str(self) -> &'static str {
        match self {
            Self::ATTRIBUTE => "an attribute",
            Self::COMMENT => "a comment",
            Self::DOC_COMMENT => "a doc comment",
            Self::EOI => "end of file",
            Self::IDENT => "an identifier",
            Self::INLINE_ATTRIBUTE => "an inline attribute",
            Self::INLINE_DOC_COMMENT => "an inline doc comment",
            Self::KW_ARGS => "`args`",
            Self::KW_CONST => "`const`",
            Self::KW_ENUM => "`enum`",
            Self::KW_ERR => "`err`",
            Self::KW_EVENT => "`event`",
            Self::KW_FALLBACK => "`fallback`",
            Self::KW_FN => "`fn`",
            Self::KW_I16 => "`i16`",
            Self::KW_I32 => "`i32`",
            Self::KW_I64 => "`i64`",
            Self::KW_I8 => "`i8`",
            Self::KW_IMPORT => "`import`",
            Self::KW_NEWTYPE => "`newtype`",
            Self::KW_OK => "`ok`",
            Self::KW_REQUIRED => "`required`",
            Self::KW_SERVICE => "`service`",
            Self::KW_STRING => "`string`",
            Self::KW_STRUCT => "`struct`",
            Self::KW_U16 => "`u16`",
            Self::KW_U32 => "`u32`",
            Self::KW_U64 => "`u64`",
            Self::KW_U8 => "`u8`",
            Self::KW_UUID => "`uuid`",
            Self::KW_VERSION => "`version`",
            Self::LIT_INT => "an integer",
            Self::LIT_STRING => "a string",
            Self::LIT_UUID => "a UUID",
            Self::PUNCT_ANG_CLOSE => "`>`",
            Self::PUNCT_ANG_OPEN => "`<`",
            Self::PUNCT_ARROW => "`->`",
            Self::PUNCT_AT => "`@`",
            Self::PUNCT_COMMA => "`,`",
            Self::PUNCT_CUR_CLOSE => "`}`",
            Self::PUNCT_CUR_OPEN => "`{`",
            Self::PUNCT_EQ => "`=`",
            Self::PUNCT_EXCLAMATION => "`!`",
            Self::PUNCT_HASH => "`#`",
            Self::PUNCT_PAR_CLOSE => "`)`",
            Self::PUNCT_PAR_OPEN => "`(`",
            Self::PUNCT_SCOPE => "`::`",
            Self::PUNCT_SEMICOLON => "`;`",
            Self::PUNCT_SQU_CLOSE => "`]`",
            Self::PUNCT_SQU_OPEN => "`[`",
            Self::TYPE => "a type",
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Found {
    Comment,
    DocComment,
    Eoi,
    Error,
    Ident,
    InlineDocComment,
    LitInt,
    LitString,
    LitStringError,
    LitUuid,
    Punct,
}

impl Found {
    fn as_label(self) -> &'static str {
        match self {
            Self::Comment => "unexpected comment",
            Self::DocComment => "unexpected doc comment",
            Self::Eoi => "unexpected end of file",
            Self::Error => "unrecognized token(s)",
            Self::Ident => "unexpected identifier",
            Self::InlineDocComment => "unexpected inline doc comment",
            Self::LitInt => "unexpected integer",
            Self::LitString => "unexpected string",
            Self::LitStringError => "found unterminated string",
            Self::LitUuid => "unexpected UUID",
            Self::Punct => "unexpected punctuation",
        }
    }
}

impl From<MaybeRef<'_, Token<'_>>> for Found {
    fn from(tok: MaybeRef<Token>) -> Self {
        match tok.into_inner() {
            Token::Comment => Self::Comment,
            Token::DocComment => Self::DocComment,
            Token::InlineDocComment => Self::InlineDocComment,
            Token::Ident(_) => Self::Ident,
            Token::LitString => Self::LitString,
            Token::LitStringError => Self::LitStringError,
            Token::LitInt => Self::LitInt,
            Token::LitUuid => Self::LitUuid,
            Token::Punct(_) => Self::Punct,
            Token::Error => Self::Error,
        }
    }
}
