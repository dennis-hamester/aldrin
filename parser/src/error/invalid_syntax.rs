use super::Error;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::grammar::Rule;
use crate::{LineCol, Parsed, Position, Span};
use std::borrow::Cow;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct InvalidSyntax {
    schema_name: String,
    pos: Position,
    expected: BTreeSet<Expected>,
}

impl InvalidSyntax {
    pub(crate) fn new<S>(schema_name: S, err: pest::error::Error<Rule>) -> Self
    where
        S: Into<String>,
    {
        use pest::error::ErrorVariant;

        let pos = Position::from_pest_error(&err);

        let positives = match err.variant {
            ErrorVariant::ParsingError { positives, .. } => positives,
            ErrorVariant::CustomError { .. } => unreachable!(),
        };

        let mut expected = BTreeSet::new();
        for rule in positives {
            Expected::add(rule, &mut expected);
        }

        Self {
            schema_name: schema_name.into(),
            pos,
            expected,
        }
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    pub fn expected(&self) -> &BTreeSet<Expected> {
        &self.expected
    }
}

impl Diagnostic for InvalidSyntax {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut reason = "expected ".to_owned();

        let mut iter = self.expected.iter().peekable();
        let mut first = true;
        let mut eof = false;
        while let Some(expected) = iter.next() {
            let expected: Cow<'static, str> = match expected {
                Expected::Eof => {
                    eof = true;
                    continue;
                }
                Expected::Ident => "an identifier".into(),
                Expected::Keyword(kw) => format!("`{kw}`").into(),
                Expected::LitInt => "an integer literal".into(),
                Expected::LitPosInt => "a positive integer literal".into(),
                Expected::LitString => "a string literal".into(),
                Expected::LitUuid => "a uuid literal".into(),
                Expected::SchemaName => "a schema name".into(),
                Expected::Token(tok) => format!("`{tok}`").into(),
            };

            if first {
                first = false;
            } else if iter.peek().is_some() || eof {
                reason.push_str(", ");
            } else {
                reason.push_str(" or ");
            }

            reason.push_str(&expected);
        }

        if eof {
            if first {
                reason.push_str("end of file");
            } else {
                reason.push_str(" or end of file");
            }
        }

        let mut fmt = Formatter::new(self, reason);

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            let span = Span {
                from: self.pos,
                to: Position {
                    index: self.pos.index + 1,
                    line_col: LineCol {
                        line: self.pos.line_col.line,
                        column: self.pos.line_col.column + 1,
                    },
                },
            };

            fmt.main_block(schema, self.pos, span, "");
        }

        fmt.format()
    }
}

impl From<InvalidSyntax> for Error {
    fn from(e: InvalidSyntax) -> Self {
        Self::InvalidSyntax(e)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expected {
    Eof,
    Ident,
    Keyword(&'static str),
    LitInt,
    LitPosInt,
    LitString,
    LitUuid,
    SchemaName,
    Token(&'static str),
}

impl Expected {
    fn add(rule: Rule, set: &mut BTreeSet<Self>) {
        const CONST_VALUE: &[Expected] = &[
            Expected::Keyword("i16"),
            Expected::Keyword("i32"),
            Expected::Keyword("i64"),
            Expected::Keyword("i8"),
            Expected::Keyword("string"),
            Expected::Keyword("u16"),
            Expected::Keyword("u32"),
            Expected::Keyword("u64"),
            Expected::Keyword("u8"),
            Expected::Keyword("uuid"),
        ];

        const DEF: &[Expected] = &[
            Expected::Keyword("const"),
            Expected::Keyword("enum"),
            Expected::Keyword("service"),
            Expected::Keyword("struct"),
            Expected::Token("#"),
        ];

        const TYPE_NAME: &[Expected] = &[
            Expected::Ident,
            Expected::Keyword("bool"),
            Expected::Keyword("bytes"),
            Expected::Keyword("f32"),
            Expected::Keyword("f64"),
            Expected::Keyword("i16"),
            Expected::Keyword("i32"),
            Expected::Keyword("i64"),
            Expected::Keyword("i8"),
            Expected::Keyword("lifetime"),
            Expected::Keyword("map"),
            Expected::Keyword("object_id"),
            Expected::Keyword("option"),
            Expected::Keyword("receiver"),
            Expected::Keyword("result"),
            Expected::Keyword("sender"),
            Expected::Keyword("service_id"),
            Expected::Keyword("set"),
            Expected::Keyword("string"),
            Expected::Keyword("u16"),
            Expected::Keyword("u32"),
            Expected::Keyword("u64"),
            Expected::Keyword("u8"),
            Expected::Keyword("unit"),
            Expected::Keyword("uuid"),
            Expected::Keyword("value"),
            Expected::Keyword("vec"),
            Expected::SchemaName,
            Expected::Token("["),
        ];

        const INLINE: &[Expected] = &[Expected::Keyword("enum"), Expected::Keyword("struct")];

        const KEY_TYPE_NAME: &[Expected] = &[
            Expected::Keyword("i16"),
            Expected::Keyword("i32"),
            Expected::Keyword("i64"),
            Expected::Keyword("i8"),
            Expected::Keyword("string"),
            Expected::Keyword("u16"),
            Expected::Keyword("u32"),
            Expected::Keyword("u64"),
            Expected::Keyword("u8"),
            Expected::Keyword("uuid"),
        ];

        const ARRAY_LEN: &[Expected] =
            &[Expected::Ident, Expected::LitPosInt, Expected::SchemaName];

        #[allow(clippy::use_self)]
        let add: &[&[Self]] = match rule {
            Rule::EOI => &[&[Expected::Eof]],
            Rule::array_len => &[ARRAY_LEN],
            Rule::const_value => &[CONST_VALUE],
            Rule::def => &[DEF],
            Rule::ident => &[&[Expected::Ident]],
            Rule::key_type_name => &[KEY_TYPE_NAME],
            Rule::kw_args => &[&[Expected::Keyword("args")]],
            Rule::kw_enum => &[&[Expected::Keyword("enum")]],
            Rule::kw_err => &[&[Expected::Keyword("err")]],
            Rule::kw_fallback => &[&[Expected::Keyword("fallback")]],
            Rule::kw_import => &[&[Expected::Keyword("import")]],
            Rule::kw_object_id => &[&[Expected::Keyword("object_id")]],
            Rule::kw_ok => &[&[Expected::Keyword("ok")]],
            Rule::kw_service_id => &[&[Expected::Keyword("service_id")]],
            Rule::kw_struct => &[&[Expected::Keyword("struct")]],
            Rule::kw_uuid => &[&[Expected::Keyword("uuid")]],
            Rule::kw_version => &[&[Expected::Keyword("version")]],
            Rule::lit_int => &[&[Expected::LitInt]],
            Rule::lit_pos_int => &[&[Expected::LitPosInt]],
            Rule::lit_string => &[&[Expected::LitString]],
            Rule::lit_uuid => &[&[Expected::LitUuid]],
            Rule::schema_name => &[&[Expected::SchemaName]],
            Rule::service_item => &[&[Expected::Keyword("fn"), Expected::Keyword("event")]],
            Rule::struct_field => &[&[Expected::Keyword("required"), Expected::Ident]],
            Rule::tok_ang_close => &[&[Expected::Token(">")]],
            Rule::tok_ang_open => &[&[Expected::Token("<")]],
            Rule::tok_arrow => &[&[Expected::Token("->")]],
            Rule::tok_at => &[&[Expected::Token("@")]],
            Rule::tok_comma => &[&[Expected::Token(",")]],
            Rule::tok_cur_close => &[&[Expected::Token("}")]],
            Rule::tok_cur_open => &[&[Expected::Token("{")]],
            Rule::tok_eq => &[&[Expected::Token("=")]],
            Rule::tok_hash => &[&[Expected::Token("#")]],
            Rule::tok_par_close => &[&[Expected::Token(")")]],
            Rule::tok_par_open => &[&[Expected::Token("(")]],
            Rule::tok_scope => &[&[Expected::Token("::")]],
            Rule::tok_squ_close => &[&[Expected::Token("]")]],
            Rule::tok_squ_open => &[&[Expected::Token("[")]],
            Rule::tok_term => &[&[Expected::Token(";")]],
            Rule::type_name => &[TYPE_NAME],
            Rule::type_name_or_inline => &[TYPE_NAME, INLINE],
            _ => return,
        };

        for &slice in add {
            set.extend(slice);
        }
    }
}
