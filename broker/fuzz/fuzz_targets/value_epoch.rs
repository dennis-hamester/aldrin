use aldrin_broker::core::ProtocolVersion;
use arbitrary::Arbitrary;

#[derive(Debug, Arbitrary)]
pub enum ValueEpoch {
    V1,
}

impl From<ValueEpoch> for ProtocolVersion {
    fn from(epoch: ValueEpoch) -> Self {
        match epoch {
            ValueEpoch::V1 => Self::V1_14,
        }
    }
}
