use super::Error;
use crate::diag::{Diagnostic, DiagnosticKind, Renderer};
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

    fn render(&self, renderer: &Renderer, parsed: &Parsed) -> String {
        let mut title = "expected ".to_owned();
        let mut iter = self.expected.iter().peekable();
        let mut first = true;
        let mut eof = false;

        while let Some(expected) = iter.next() {
            let expected: Cow<'static, str> = match expected {
                Expected::Attribute => "an attribute".into(),
                Expected::DocString => "a doc string".into(),
                Expected::DocStringInline => "an inline doc string".into(),

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
                title.push_str(", ");
            } else {
                title.push_str(" or ");
            }

            title.push_str(&expected);
        }

        if eof {
            if first {
                title.push_str("end of file");
            } else {
                title.push_str(" or end of file");
            }
        }

        let mut report = renderer.error(title);

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            let width = schema.source().unwrap()[self.pos.index..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(1);

            let span = Span {
                from: self.pos,
                to: Position {
                    index: self.pos.index + width,
                    line_col: LineCol {
                        line: self.pos.line_col.line,
                        column: self.pos.line_col.column + width,
                    },
                },
            };

            report = report.snippet(schema, span, "");
        }

        report.render()
    }
}

impl From<InvalidSyntax> for Error {
    fn from(e: InvalidSyntax) -> Self {
        Self::InvalidSyntax(e)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expected {
    Attribute,
    DocString,
    DocStringInline,
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
            Expected::Attribute,
            Expected::DocString,
            Expected::Keyword("const"),
            Expected::Keyword("enum"),
            Expected::Keyword("newtype"),
            Expected::Keyword("service"),
            Expected::Keyword("struct"),
        ];

        const SERVICE_ITEM: &[Expected] = &[
            Expected::DocString,
            Expected::Keyword("event"),
            Expected::Keyword("fn"),
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

        const ARRAY_LEN: &[Expected] =
            &[Expected::Ident, Expected::LitPosInt, Expected::SchemaName];

        #[allow(clippy::use_self)]
        let add: &[&[Self]] = match rule {
            Rule::EOI => &[&[Expected::Eof]],
            Rule::array_len => &[ARRAY_LEN],
            Rule::const_value => &[CONST_VALUE],
            Rule::def => &[DEF],
            Rule::doc_string => &[&[Expected::DocString]],
            Rule::doc_string_inline => &[&[Expected::DocStringInline]],
            Rule::event_fallback => &[&[Expected::DocString, Expected::Keyword("event")]],
            Rule::fn_fallback => &[&[Expected::DocString, Expected::Keyword("fn")]],
            Rule::ident => &[&[Expected::Ident]],
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
            Rule::service_item | Rule::service_fallback => &[SERVICE_ITEM],
            Rule::struct_field => &[&[Expected::Keyword("required"), Expected::Ident]],
            Rule::tok_ang_close => &[&[Expected::Token(">")]],
            Rule::tok_ang_open => &[&[Expected::Token("<")]],
            Rule::tok_arrow => &[&[Expected::Token("->")]],
            Rule::tok_at => &[&[Expected::Token("@")]],
            Rule::tok_comma => &[&[Expected::Token(",")]],
            Rule::tok_cur_close => &[&[Expected::Token("}")]],
            Rule::tok_cur_open => &[&[Expected::Token("{")]],
            Rule::tok_eq => &[&[Expected::Token("=")]],
            Rule::tok_hash => &[&[Expected::Attribute]],
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
