use super::Error;
use crate::grammar::Rule;
use crate::Position;
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

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    pub fn expected(&self) -> &HashSet<Expected> {
        &self.expected
    }
}

impl From<InvalidSyntax> for Error {
    fn from(e: InvalidSyntax) -> Self {
        Error::InvalidSyntax(e)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expected {
    Attribute,
    ConstDef,
    ConstValue,
    EnumDef,
    EnumVariant,
    EnumVariantType,
    Eof,
    Ident,
    ImportStmt,
    InlineEnum,
    InlineStruct,
    KeyTypeName,
    LitInt,
    LitPosInt,
    LitString,
    LitUuid,
    SchemaName,
    ServiceDef,
    ServiceUuid,
    ServiceVersion,
    StructDef,
    StructField,
    TokenAngClose,
    TokenAngOpen,
    TokenArrow,
    TokenAt,
    TokenComma,
    TokenCurlyClose,
    TokenCurlyOpen,
    TokenEquals,
    TokenParClose,
    TokenParOpen,
    TokenScope,
    TokenSquareClose,
    TokenSquareOpen,
    TokenTerm,
    TypeName,
}

impl Expected {
    fn add(rule: Rule, set: &mut HashSet<Self>) {
        match rule {
            Rule::EOI => set.insert(Expected::Eof),
            Rule::attribute | Rule::tok_hash => set.insert(Expected::Attribute),
            Rule::const_def | Rule::kw_const => set.insert(Expected::ConstDef),
            Rule::const_value => set.insert(Expected::ConstValue),
            Rule::def => {
                set.insert(Expected::StructDef);
                set.insert(Expected::EnumDef);
                set.insert(Expected::ServiceDef);
                set.insert(Expected::ConstDef)
            }
            Rule::enum_inline => set.insert(Expected::InlineEnum),
            Rule::enum_variant => set.insert(Expected::EnumVariant),
            Rule::enum_variant_type => set.insert(Expected::EnumVariantType),
            Rule::ident => set.insert(Expected::Ident),
            Rule::import_stmt | Rule::kw_import => set.insert(Expected::ImportStmt),
            Rule::key_type_name => set.insert(Expected::KeyTypeName),
            Rule::kw_enum | Rule::enum_def => set.insert(Expected::EnumDef),
            Rule::kw_service | Rule::service_def => set.insert(Expected::ServiceDef),
            Rule::kw_struct | Rule::struct_def => set.insert(Expected::StructDef),
            Rule::kw_version | Rule::service_version => set.insert(Expected::ServiceVersion),
            Rule::lit_int => set.insert(Expected::LitInt),
            Rule::lit_pos_int => set.insert(Expected::LitPosInt),
            Rule::lit_string => set.insert(Expected::LitString),
            Rule::lit_uuid => set.insert(Expected::LitUuid),
            Rule::schema_name => set.insert(Expected::SchemaName),
            Rule::service_uuid => set.insert(Expected::ServiceUuid),
            Rule::struct_field => set.insert(Expected::StructField),
            Rule::struct_inline => set.insert(Expected::InlineStruct),
            Rule::tok_ang_close => set.insert(Expected::TokenAngClose),
            Rule::tok_ang_open => set.insert(Expected::TokenAngOpen),
            Rule::tok_arrow => set.insert(Expected::TokenArrow),
            Rule::tok_at => set.insert(Expected::TokenAt),
            Rule::tok_comma => set.insert(Expected::TokenComma),
            Rule::tok_cur_close => set.insert(Expected::TokenCurlyClose),
            Rule::tok_cur_open => set.insert(Expected::TokenCurlyOpen),
            Rule::tok_eq => set.insert(Expected::TokenEquals),
            Rule::tok_par_close => set.insert(Expected::TokenParClose),
            Rule::tok_par_open => set.insert(Expected::TokenParOpen),
            Rule::tok_scope => set.insert(Expected::TokenScope),
            Rule::tok_squ_close => set.insert(Expected::TokenSquareClose),
            Rule::tok_squ_open => set.insert(Expected::TokenSquareOpen),
            Rule::tok_term => set.insert(Expected::TokenTerm),
            Rule::type_name => set.insert(Expected::TypeName),
            Rule::type_name_or_inline => {
                set.insert(Expected::TypeName);
                set.insert(Expected::InlineStruct);
                set.insert(Expected::InlineEnum)
            }

            Rule::COMMENT
            | Rule::WHITESPACE
            | Rule::const_i16
            | Rule::const_i32
            | Rule::const_i64
            | Rule::const_i8
            | Rule::const_string
            | Rule::const_u16
            | Rule::const_u32
            | Rule::const_u64
            | Rule::const_u8
            | Rule::const_uuid
            | Rule::external_type_name
            | Rule::file
            | Rule::kw_bool
            | Rule::kw_bytes
            | Rule::kw_f32
            | Rule::kw_f64
            | Rule::kw_i16
            | Rule::kw_i32
            | Rule::kw_i64
            | Rule::kw_i8
            | Rule::kw_map
            | Rule::kw_optional
            | Rule::kw_required
            | Rule::kw_set
            | Rule::kw_string
            | Rule::kw_u16
            | Rule::kw_u32
            | Rule::kw_u64
            | Rule::kw_u8
            | Rule::kw_uuid
            | Rule::kw_value
            | Rule::kw_vec
            | Rule::lit_pos_nonzero_int
            | Rule::lit_string_char
            | Rule::map_type
            | Rule::set_type
            | Rule::vec_type
            | Rule::ws => false,
        };
    }
}
