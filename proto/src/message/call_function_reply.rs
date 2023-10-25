use super::message_ops::Sealed;
use super::{
    Message, MessageDeserializeError, MessageKind, MessageOps, MessageSerializeError,
    MessageSerializer, MessageWithValueDeserializer,
};
use crate::error::SerializeError;
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::value_serializer::Serialize;
use bytes::BytesMut;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum CallFunctionReplyKind {
    Ok = 0,
    Err = 1,
    Aborted = 2,
    InvalidService = 3,
    InvalidFunction = 4,
    InvalidArgs = 5,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub enum CallFunctionResult {
    Ok(SerializedValue),
    Err(SerializedValue),
    Aborted,
    InvalidService,
    InvalidFunction,
    InvalidArgs,
}

impl CallFunctionResult {
    pub fn ok_with_serialize_value<T: Serialize + ?Sized>(
        value: &T,
    ) -> Result<Self, SerializeError> {
        SerializedValue::serialize(value).map(Self::Ok)
    }

    pub fn err_with_serialize_value<T: Serialize + ?Sized>(
        value: &T,
    ) -> Result<Self, SerializeError> {
        SerializedValue::serialize(value).map(Self::Err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct CallFunctionReply {
    pub serial: u32,
    pub result: CallFunctionResult,
}

impl CallFunctionReply {
    pub fn ok_with_serialize_value<T: Serialize + ?Sized>(
        serial: u32,
        value: &T,
    ) -> Result<Self, SerializeError> {
        let result = CallFunctionResult::ok_with_serialize_value(value)?;
        Ok(Self { serial, result })
    }

    pub fn err_with_serialize_value<T: Serialize + ?Sized>(
        serial: u32,
        value: &T,
    ) -> Result<Self, SerializeError> {
        let result = CallFunctionResult::err_with_serialize_value(value)?;
        Ok(Self { serial, result })
    }
}

impl MessageOps for CallFunctionReply {
    fn kind(&self) -> MessageKind {
        MessageKind::CallFunctionReply
    }

    fn serialize_message(self) -> Result<BytesMut, MessageSerializeError> {
        let serializer = match self.result {
            CallFunctionResult::Ok(value) => {
                let mut serializer =
                    MessageSerializer::with_value(value, MessageKind::CallFunctionReply)?;
                serializer.put_varint_u32_le(self.serial);
                serializer.put_discriminant_u8(CallFunctionReplyKind::Ok);
                serializer
            }

            CallFunctionResult::Err(value) => {
                let mut serializer =
                    MessageSerializer::with_value(value, MessageKind::CallFunctionReply)?;
                serializer.put_varint_u32_le(self.serial);
                serializer.put_discriminant_u8(CallFunctionReplyKind::Err);
                serializer
            }

            CallFunctionResult::Aborted => {
                let mut serializer =
                    MessageSerializer::with_none_value(MessageKind::CallFunctionReply);
                serializer.put_varint_u32_le(self.serial);
                serializer.put_discriminant_u8(CallFunctionReplyKind::Aborted);
                serializer
            }

            CallFunctionResult::InvalidService => {
                let mut serializer =
                    MessageSerializer::with_none_value(MessageKind::CallFunctionReply);
                serializer.put_varint_u32_le(self.serial);
                serializer.put_discriminant_u8(CallFunctionReplyKind::InvalidService);
                serializer
            }

            CallFunctionResult::InvalidFunction => {
                let mut serializer =
                    MessageSerializer::with_none_value(MessageKind::CallFunctionReply);
                serializer.put_varint_u32_le(self.serial);
                serializer.put_discriminant_u8(CallFunctionReplyKind::InvalidFunction);
                serializer
            }

            CallFunctionResult::InvalidArgs => {
                let mut serializer =
                    MessageSerializer::with_none_value(MessageKind::CallFunctionReply);
                serializer.put_varint_u32_le(self.serial);
                serializer.put_discriminant_u8(CallFunctionReplyKind::InvalidArgs);
                serializer
            }
        };

        serializer.finish()
    }

    fn deserialize_message(buf: BytesMut) -> Result<Self, MessageDeserializeError> {
        let mut deserializer =
            MessageWithValueDeserializer::new(buf, MessageKind::CallFunctionReply)?;

        let serial = deserializer.try_get_varint_u32_le()?;

        match deserializer.try_get_discriminant_u8()? {
            CallFunctionReplyKind::Ok => {
                let value = deserializer.finish()?;
                Ok(Self {
                    serial,
                    result: CallFunctionResult::Ok(value),
                })
            }

            CallFunctionReplyKind::Err => {
                let value = deserializer.finish()?;
                Ok(Self {
                    serial,
                    result: CallFunctionResult::Err(value),
                })
            }

            CallFunctionReplyKind::Aborted => {
                deserializer.finish_discard_value()?;
                Ok(Self {
                    serial,
                    result: CallFunctionResult::Aborted,
                })
            }

            CallFunctionReplyKind::InvalidService => {
                deserializer.finish_discard_value()?;
                Ok(Self {
                    serial,
                    result: CallFunctionResult::InvalidService,
                })
            }

            CallFunctionReplyKind::InvalidFunction => {
                deserializer.finish_discard_value()?;
                Ok(Self {
                    serial,
                    result: CallFunctionResult::InvalidFunction,
                })
            }

            CallFunctionReplyKind::InvalidArgs => {
                deserializer.finish_discard_value()?;
                Ok(Self {
                    serial,
                    result: CallFunctionResult::InvalidArgs,
                })
            }
        }
    }

    fn value(&self) -> Option<&SerializedValueSlice> {
        match self.result {
            CallFunctionResult::Ok(ref value) | CallFunctionResult::Err(ref value) => Some(value),

            CallFunctionResult::Aborted
            | CallFunctionResult::InvalidService
            | CallFunctionResult::InvalidFunction
            | CallFunctionResult::InvalidArgs => None,
        }
    }
}

impl Sealed for CallFunctionReply {}

impl From<CallFunctionReply> for Message {
    fn from(msg: CallFunctionReply) -> Self {
        Self::CallFunctionReply(msg)
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{
        assert_deserialize_eq, assert_deserialize_eq_with_value, assert_serialize_eq,
    };
    use super::super::Message;
    use super::{CallFunctionReply, CallFunctionResult};

    #[test]
    fn ok() {
        let serialized = [13, 0, 0, 0, 12, 2, 0, 0, 0, 3, 4, 1, 0];
        let value = 4u8;

        let msg = CallFunctionReply::ok_with_serialize_value(1, &value).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::CallFunctionReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }

    #[test]
    fn err() {
        let serialized = [13, 0, 0, 0, 12, 2, 0, 0, 0, 3, 4, 1, 1];
        let value = 4u8;

        let msg = CallFunctionReply::err_with_serialize_value(1, &value).unwrap();
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);

        let msg = Message::CallFunctionReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq_with_value(&msg, serialized, &value);
    }

    #[test]
    fn aborted() {
        let serialized = [12, 0, 0, 0, 12, 1, 0, 0, 0, 0, 1, 2];

        let msg = CallFunctionReply {
            serial: 1,
            result: CallFunctionResult::Aborted,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CallFunctionReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_service() {
        let serialized = [12, 0, 0, 0, 12, 1, 0, 0, 0, 0, 1, 3];

        let msg = CallFunctionReply {
            serial: 1,
            result: CallFunctionResult::InvalidService,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CallFunctionReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_function() {
        let serialized = [12, 0, 0, 0, 12, 1, 0, 0, 0, 0, 1, 4];

        let msg = CallFunctionReply {
            serial: 1,
            result: CallFunctionResult::InvalidFunction,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CallFunctionReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }

    #[test]
    fn invalid_args() {
        let serialized = [12, 0, 0, 0, 12, 1, 0, 0, 0, 0, 1, 5];

        let msg = CallFunctionReply {
            serial: 1,
            result: CallFunctionResult::InvalidArgs,
        };
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);

        let msg = Message::CallFunctionReply(msg);
        assert_serialize_eq(&msg, serialized);
        assert_deserialize_eq(&msg, serialized);
    }
}
