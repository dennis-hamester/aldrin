use crate::message::MessageOps;
use crate::{ProtocolVersion, SerializedValue, SerializedValueSlice};
use std::borrow::Cow;
use thiserror::Error;

pub(crate) fn convert(
    value: &SerializedValueSlice,
    from: Option<ProtocolVersion>,
    to: ProtocolVersion,
) -> Result<Cow<SerializedValueSlice>, ValueConversionError> {
    const MAX: ProtocolVersion = ProtocolVersion::V1_19;

    let from = Epoch::try_from(from.unwrap_or(MAX))?;
    let to = Epoch::try_from(to)?;

    if from == to {
        Ok(Cow::Borrowed(value))
    } else {
        unreachable!()
    }
}

pub(crate) fn convert_mut(
    value: &mut SerializedValue,
    from: Option<ProtocolVersion>,
    to: ProtocolVersion,
) -> Result<(), ValueConversionError> {
    match convert(value, from, to)? {
        Cow::Owned(converted) => {
            *value = converted;
            Ok(())
        }

        Cow::Borrowed(_) => Ok(()),
    }
}

pub(crate) fn convert_in_message(
    msg: &mut impl MessageOps,
    from: Option<ProtocolVersion>,
    to: ProtocolVersion,
) -> Result<(), ValueConversionError> {
    match msg.value_mut() {
        Some(value) => convert_mut(value, from, to),
        None => Ok(()),
    }
}

/// Error when converting a value.
#[derive(Error, Debug, Clone)]
pub enum ValueConversionError {
    /// The requested version is invalid.
    #[error("invalid protocol version")]
    InvalidVersion,
}

#[derive(PartialEq, Eq)]
enum Epoch {
    V1,
}

impl TryFrom<ProtocolVersion> for Epoch {
    type Error = ValueConversionError;

    fn try_from(version: ProtocolVersion) -> Result<Self, Self::Error> {
        const V1_MIN: ProtocolVersion = ProtocolVersion::V1_14;
        const V1_MAX: ProtocolVersion = ProtocolVersion::V1_19;

        if (version >= V1_MIN) && (version <= V1_MAX) {
            Ok(Self::V1)
        } else {
            Err(ValueConversionError::InvalidVersion)
        }
    }
}
