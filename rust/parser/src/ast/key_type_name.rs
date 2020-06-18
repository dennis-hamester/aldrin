use crate::grammar::Rule;
use crate::Span;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct KeyTypeName {
    span: Span,
    kind: KeyTypeNameKind,
}

impl KeyTypeName {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::key_type_name);

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        let kind = KeyTypeNameKind::parse(pair);

        KeyTypeName { span, kind }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn kind(&self) -> &KeyTypeNameKind {
        &self.kind
    }
}

#[derive(Debug, Clone)]
pub enum KeyTypeNameKind {
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

impl KeyTypeNameKind {
    fn parse(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::kw_u8 => KeyTypeNameKind::U8,
            Rule::kw_i8 => KeyTypeNameKind::I8,
            Rule::kw_u16 => KeyTypeNameKind::U16,
            Rule::kw_i16 => KeyTypeNameKind::I16,
            Rule::kw_u32 => KeyTypeNameKind::U32,
            Rule::kw_i32 => KeyTypeNameKind::I32,
            Rule::kw_u64 => KeyTypeNameKind::U64,
            Rule::kw_i64 => KeyTypeNameKind::I64,
            Rule::kw_string => KeyTypeNameKind::String,
            Rule::kw_uuid => KeyTypeNameKind::Uuid,
            _ => unreachable!(),
        }
    }
}
