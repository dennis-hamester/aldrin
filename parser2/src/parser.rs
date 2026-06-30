use crate::error::IoError;
use crate::issues::Issues;
use crate::lexer::Lexer;
use crate::validate::Validate;
use crate::{Error, Resolver, Schema, Span, Spanned, Warning};
use chumsky::Parser as _;
use chumsky::input::{Input, Stream};
use indexmap::IndexMap;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Parser {
    main_schema_ref: SchemaRef,
    schemas: IndexMap<String, ParserEntry>,
    issues: Issues,
}

impl Parser {
    pub fn new(mut resolver: impl Resolver) -> Self {
        let mut schemas = IndexMap::new();
        let mut issues = Issues::default();

        let (main_schema_name, main_schema) = resolver.main_schema();
        let main_schema_name = main_schema_name.into_owned();
        let main_schema_ref = SchemaRef(0);

        schemas.insert(
            main_schema_name.clone(),
            ParserEntry::empty(main_schema_ref, main_schema_name, main_schema.path),
        );

        let mut pending = VecDeque::from([(main_schema_ref, main_schema.source)]);

        while let Some((schema_ref, source)) = pending.pop_front() {
            let source = match source {
                Ok(source) => source,

                Err(e) => {
                    issues.add_error(IoError::new(schema_ref, e));
                    continue;
                }
            };

            let source_trimmed = source.trim_end();

            let input = Stream::from_iter(Lexer::new(source_trimmed)).map(
                Span::new(source_trimmed.len(), source_trimmed.len()),
                |(tok, span)| (tok, span),
            );

            let res = Schema::parser().parse(input).into_result();

            let schema = match res {
                Ok(schema) => schema,

                Err(errs) => {
                    for mut e in errs {
                        e.set_schema(schema_ref);
                        issues.add_error(e);
                    }

                    let (_, entry) = schemas.get_index_mut(schema_ref.0).unwrap();
                    entry.source = Some(source);
                    continue;
                }
            };

            for import in &schema.imports {
                let name_span = import.schema.span;
                let name = &source[name_span.start..name_span.end];

                if schemas.contains_key(name) {
                    continue;
                }

                if let Some(file) = resolver.resolve(name) {
                    let schema_ref = SchemaRef(schemas.len());

                    schemas.insert(
                        name.to_owned(),
                        ParserEntry::empty(schema_ref, name.to_owned(), file.path),
                    );

                    pending.push_back((schema_ref, file.source));
                }
            }

            let (_, entry) = schemas.get_index_mut(schema_ref.0).unwrap();
            entry.source = Some(source);
            entry.schema = Some(schema);
        }

        for entry in schemas.values() {
            let schema_ref = entry.schema_ref();

            if entry.schema().is_some() {
                Validate::run(
                    schema_ref,
                    &schemas,
                    schema_ref == main_schema_ref,
                    &mut issues,
                );
            }
        }

        Self {
            main_schema_ref,
            schemas,
            issues,
        }
    }

    pub fn main_schema_ref(&self) -> SchemaRef {
        self.main_schema_ref
    }

    pub fn main_schema(&self) -> &ParserEntry {
        self.schema(self.main_schema_ref)
    }

    pub fn schema(&self, schema_ref: SchemaRef) -> &ParserEntry {
        self.schemas.get_index(schema_ref.0).unwrap().1
    }

    pub fn schema_by_name(&self, name: &str) -> Option<&ParserEntry> {
        self.schemas.get(name)
    }

    pub fn errors(&self) -> &[Error] {
        self.issues.errors()
    }

    pub fn warnings(&self) -> &[Warning] {
        self.issues.warnings()
    }

    pub fn other_warnings(&self) -> &[Warning] {
        self.issues.other_warnings()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SchemaRef(pub(crate) usize);

impl SchemaRef {
    pub(crate) const DUMMY: Self = Self(usize::MAX);
}

#[derive(Debug)]
pub struct ParserEntry {
    schema_ref: SchemaRef,
    name: String,
    path: String,
    source: Option<String>,
    schema: Option<Schema>,
}

impl ParserEntry {
    fn empty(schema_ref: SchemaRef, name: String, path: String) -> Self {
        Self {
            schema_ref,
            name,
            path,
            source: None,
            schema: None,
        }
    }

    pub fn schema_ref(&self) -> SchemaRef {
        self.schema_ref
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn take_source(&mut self) -> Option<String> {
        self.source.take()
    }

    pub fn source_of(&self, spanned: &impl Spanned) -> Option<&str> {
        let source = self.source()?;
        Some(spanned.source(source))
    }

    pub fn schema(&self) -> Option<&Schema> {
        self.schema.as_ref()
    }

    pub fn take_schema(&mut self) -> Option<Schema> {
        self.schema.take()
    }
}
