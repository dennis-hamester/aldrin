use super::Error;
use crate::ast::Ident;
use crate::validate::Validate;

const KEYWORDS: &[&str] = &[
    "import", "struct", "enum", "service", "fn", "event", "const",
];

#[derive(Debug)]
pub struct KeywordAsIdent {
    schema_name: String,
    ident: Ident,
}

impl KeywordAsIdent {
    pub(crate) fn validate(ident: &Ident, validate: &mut Validate) {
        if !KEYWORDS.contains(&ident.value()) {
            return;
        }

        validate.add_error(KeywordAsIdent {
            schema_name: validate.schema_name().to_owned(),
            ident: ident.clone(),
        });
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl From<KeywordAsIdent> for Error {
    fn from(e: KeywordAsIdent) -> Self {
        Error::KeywordAsIdent(e)
    }
}
