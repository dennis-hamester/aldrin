use serde::Deserialize;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Default, Eq, Deserialize)]
#[serde(from = "String")]
pub(crate) struct ClientId(Option<String>);

impl ClientId {
    pub(crate) fn get(&self) -> &str {
        self.0.as_deref().unwrap_or("default")
    }
}

impl PartialEq for ClientId {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl Hash for ClientId {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.get().hash(state)
    }
}

impl From<String> for ClientId {
    fn from(s: String) -> Self {
        Self(Some(s))
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.get().fmt(f)
    }
}
