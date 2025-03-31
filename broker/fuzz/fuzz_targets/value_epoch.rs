use aldrin_broker::core::ProtocolVersion;
use arbitrary::Arbitrary;

#[derive(Debug, Arbitrary)]
pub enum ValueEpoch {
    V1,
    V2,
}

impl From<ValueEpoch> for ProtocolVersion {
    fn from(epoch: ValueEpoch) -> Self {
        match epoch {
            ValueEpoch::V1 => Self::V1_14,
            ValueEpoch::V2 => Self::V1_20,
        }
    }
}
