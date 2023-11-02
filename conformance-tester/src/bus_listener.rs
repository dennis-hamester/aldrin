use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BusListenerScope {
    Current,
    New,
    All,
}

impl From<aldrin_proto::BusListenerScope> for BusListenerScope {
    fn from(scope: aldrin_proto::BusListenerScope) -> Self {
        match scope {
            aldrin_proto::BusListenerScope::Current => Self::Current,
            aldrin_proto::BusListenerScope::New => Self::New,
            aldrin_proto::BusListenerScope::All => Self::All,
        }
    }
}

impl From<BusListenerScope> for aldrin_proto::BusListenerScope {
    fn from(scope: BusListenerScope) -> Self {
        match scope {
            BusListenerScope::Current => Self::Current,
            BusListenerScope::New => Self::New,
            BusListenerScope::All => Self::All,
        }
    }
}
