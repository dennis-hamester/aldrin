use super::{Comment, Definition, DocComment, Import, Prelude};
use crate::error::ParseError;
use crate::lexer::Token;
use crate::{Span, Visitor};
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::prelude::group;
use chumsky::{IterParser, Parser};
use std::ops::ControlFlow;

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

    pub fn visit<'a, T: Visitor<'a>>(&'a self, mut visitor: T) -> Option<T::Output> {
        self.visit_impl(&mut visitor).break_value()
    }

    fn visit_impl<'a, T: Visitor<'a> + ?Sized>(
        &'a self,
        visitor: &mut T,
    ) -> ControlFlow<T::Output> {
        visitor.schema(self)?;

        for import in &self.imports {
            import.visit_impl(visitor, self)?;
        }

        for def in &self.definitions {
            def.visit_impl(visitor, self)?;
        }

        ControlFlow::Continue(())
    }
}
