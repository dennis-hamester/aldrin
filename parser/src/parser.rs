use crate::error::DuplicateServiceUuid;
use crate::issues::Issues;
use crate::validate::Validate;
use crate::{Error, Resolver, Schema, Warning};
use std::collections::hash_map::{Entry, HashMap};
use std::iter;

#[derive(Debug)]
pub struct Parser {
    main_schema: String,
    schemas: HashMap<String, Schema>,
    issues: Issues,
}

impl Parser {
    pub fn parse<T: Resolver>(mut resolver: T) -> Self {
        let mut issues = Issues::default();
        let main_schema = Schema::parse(&resolver.main_schema(), &mut issues);

        let mut this = Self {
            main_schema: main_schema.name().to_owned(),
            schemas: HashMap::from_iter(iter::once((main_schema.name().to_owned(), main_schema))),
            issues,
        };

        let mut imports = this
            .main_schema()
            .imports()
            .iter()
            .map(|i| i.schema_name().value().to_owned())
            .collect::<Vec<_>>();

        while let Some(import) = imports.pop() {
            let Entry::Vacant(entry) = this.schemas.entry(import) else {
                continue;
            };

            let Some(schema_file) = resolver.resolve(entry.key()) else {
                continue;
            };

            let schema = Schema::parse(&schema_file, &mut this.issues);

            imports.extend(
                schema
                    .imports()
                    .iter()
                    .map(|i| i.schema_name().value().to_owned()),
            );

            entry.insert(schema);
        }

        this.validate();
        this
    }

    fn validate(&mut self) {
        DuplicateServiceUuid::validate(self.schemas.values(), &mut self.issues);

        for (schema_name, schema) in &self.schemas {
            let is_main_schema = *schema_name == self.main_schema;

            let mut validate =
                Validate::new(schema_name, &mut self.issues, &self.schemas, is_main_schema);

            schema.validate(&mut validate);
        }
    }

    pub fn main_schema(&self) -> &Schema {
        self.get_schema(&self.main_schema).unwrap()
    }

    pub fn get_schema(&self, schema_name: &str) -> Option<&Schema> {
        self.schemas.get(schema_name)
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

    pub(crate) fn schemas(&self) -> &HashMap<String, Schema> {
        &self.schemas
    }
}
