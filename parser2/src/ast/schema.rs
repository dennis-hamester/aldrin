use super::{Comment, Definition, DocComment, Import, Prelude};
use crate::Span;
use crate::error::ParseError;
use crate::lexer::Token;
use crate::validate::Validate;
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::prelude::group;
use chumsky::{IterParser, Parser};

#[derive(Debug, Clone)]
pub struct Schema {
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub imports: Vec<Import>,
    pub definitions: Vec<Definition>,
}

impl Schema {
    pub(crate) fn parser<'a, I>() -> impl Parser<'a, I, Self, Err<ParseError>>
    where
        I: ValueInput<'a, Token = Token<'a>, Span = Span>,
    {
        group((
            Prelude::schema_parser(),
            Import::parser().repeated().collect(),
            Definition::parser().repeated().collect(),
        ))
        .map(|(prelude, imports, definitions)| Self {
            comments: prelude.comments,
            doc_comments: prelude.doc_comments,
            imports,
            definitions,
        })
    }

    #[expect(clippy::unused_self)]
    pub(crate) fn validate(&self, _validate: &mut Validate) {}
}
