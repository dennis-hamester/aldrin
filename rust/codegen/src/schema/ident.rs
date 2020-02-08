use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Ident(pub String);

impl Ident {
    pub fn from_string<S>(s: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Ok(Ident(s.into()))
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct ModuleName(pub String);

impl ModuleName {
    pub fn from_string<S>(s: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Ok(ModuleName(s.into()))
    }
}
