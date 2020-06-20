use super::Error;
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::grammar::Rule;
use crate::{Parsed, Position};
use std::collections::HashSet;

#[derive(Debug)]
pub struct InvalidSyntax {
    schema_name: String,
    pos: Position,
    expected: HashSet<Expected>,
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

        let mut expected = HashSet::with_capacity(positives.len());
        for rule in positives {
            Expected::add(rule, &mut expected);
        }

        InvalidSyntax {
            schema_name: schema_name.into(),
            pos,
            expected,
        }
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    pub fn expected(&self) -> &HashSet<Expected> {
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
        todo!()
    }
}

impl From<InvalidSyntax> for Error {
    fn from(e: InvalidSyntax) -> Self {
        Error::InvalidSyntax(e)
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
    fn add(rule: Rule, set: &mut HashSet<Self>) {
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
            Expected::Keyword("map"),
            Expected::Keyword("set"),
            Expected::Keyword("string"),
            Expected::Keyword("u16"),
            Expected::Keyword("u32"),
            Expected::Keyword("u64"),
            Expected::Keyword("u8"),
            Expected::Keyword("uuid"),
            Expected::Keyword("value"),
            Expected::Keyword("vec"),
            Expected::SchemaName,
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

        let add: &[&[Expected]] = match rule {
            Rule::EOI => &[&[Expected::Eof]],
            Rule::const_value => &[CONST_VALUE],
            Rule::def => &[DEF],
            Rule::enum_variant_type => &[TYPE_NAME, INLINE, &[Expected::Keyword("optional")]],
            Rule::event_type => &[TYPE_NAME, INLINE, &[Expected::Keyword("optional")]],
            Rule::ident => &[&[Expected::Ident]],
            Rule::key_type_name => &[KEY_TYPE_NAME],
            Rule::kw_args => &[&[Expected::Keyword("args")]],
            Rule::kw_enum => &[&[Expected::Keyword("enum")]],
            Rule::kw_err => &[&[Expected::Keyword("err")]],
            Rule::kw_import => &[&[Expected::Keyword("import")]],
            Rule::kw_ok => &[&[Expected::Keyword("ok")]],
            Rule::kw_optional => &[&[Expected::Keyword("optional")]],
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
