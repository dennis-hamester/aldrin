use super::Error;
use crate::ast::{Ident, LitUuid};
use crate::diag::{Diagnostic, DiagnosticKind, Formatted, Formatter};
use crate::issues::Issues;
use crate::{Parsed, Schema};
use std::collections::HashMap;

#[derive(Debug)]
pub struct DuplicateServiceUuid {
    schema_name: String,
    uuid: LitUuid,
    svc_idents: Vec<(String, Ident)>,
}

impl DuplicateServiceUuid {
    pub(crate) fn validate<'a, I>(schemas: I, issues: &mut Issues)
    where
        I: IntoIterator<Item = &'a Schema>,
    {
        let mut uuids: HashMap<_, Vec<_>> = HashMap::new();

        for schema in schemas {
            for def in schema.definitions() {
                let Some(svc) = def.as_service() else {
                    continue;
                };

                uuids
                    .entry(svc.uuid().value())
                    .or_default()
                    .push((schema, svc));
            }
        }

        for (_, entries) in uuids {
            if entries.len() > 1 {
                let first = entries.first().unwrap();

                issues.add_error(Self {
                    schema_name: first.0.name().to_owned(),
                    uuid: first.1.uuid().clone(),
                    svc_idents: entries
                        .into_iter()
                        .map(|(schema, svc)| (schema.name().to_owned(), svc.name().clone()))
                        .collect(),
                });
            }
        }
    }

    pub fn uuid(&self) -> &LitUuid {
        &self.uuid
    }

    pub fn service_idents(&self) -> &[(String, Ident)] {
        &self.svc_idents
    }
}

impl Diagnostic for DuplicateServiceUuid {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn schema_name(&self) -> &str {
        &self.schema_name
    }

    fn format<'a>(&'a self, parsed: &'a Parsed) -> Formatted<'a> {
        let mut fmt = Formatter::new(
            self,
            format!("duplicate service uuid `{}`", self.uuid.value()),
        );

        if let Some(schema) = parsed.get_schema(&self.schema_name) {
            fmt.main_block(
                schema,
                self.uuid.span().from,
                self.uuid.span(),
                "this uuid is used multiple times",
            );
        }

        for (schema_name, svc_ident) in &self.svc_idents {
            if let Some(schema) = parsed.get_schema(schema_name) {
                fmt.info_block(
                    schema,
                    svc_ident.span().from,
                    svc_ident.span(),
                    "used for this service",
                );
            }
        }

        fmt.help("use different uuids for each service");
        fmt.format()
    }
}

impl From<DuplicateServiceUuid> for Error {
    fn from(e: DuplicateServiceUuid) -> Self {
        Self::DuplicateServiceUuid(e)
    }
}
