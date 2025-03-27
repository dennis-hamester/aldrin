use aldrin_core::message::{Message, MessageOps};
use aldrin_core::{ProtocolVersion, ValueConversionError};

pub(crate) struct VersionedMessage {
    pub msg: Message,
    pub version: Option<ProtocolVersion>,
}

impl VersionedMessage {
    pub fn new(msg: impl Into<Message>, version: Option<ProtocolVersion>) -> Self {
        Self {
            msg: msg.into(),
            version,
        }
    }

    pub fn with_version(msg: impl Into<Message>, version: ProtocolVersion) -> Self {
        Self::new(msg, Some(version))
    }

    pub fn convert_value(mut self, to: ProtocolVersion) -> Result<Message, ValueConversionError> {
        self.msg.convert_value(self.version, to)?;
        Ok(self.msg)
    }
}

impl<T> From<T> for VersionedMessage
where
    T: Into<Message>,
{
    fn from(msg: T) -> Self {
        Self::new(msg.into(), None)
    }
}
