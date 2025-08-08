use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum BusListenerScope {
    Current,
    New,
    All,
}

impl From<aldrin_core::BusListenerScope> for BusListenerScope {
    fn from(scope: aldrin_core::BusListenerScope) -> Self {
        match scope {
            aldrin_core::BusListenerScope::Current => Self::Current,
            aldrin_core::BusListenerScope::New => Self::New,
            aldrin_core::BusListenerScope::All => Self::All,
        }
    }
}

impl From<BusListenerScope> for aldrin_core::BusListenerScope {
    fn from(scope: BusListenerScope) -> Self {
        match scope {
            BusListenerScope::Current => Self::Current,
            BusListenerScope::New => Self::New,
            BusListenerScope::All => Self::All,
        }
    }
}
