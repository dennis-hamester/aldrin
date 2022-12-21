use super::{Message, MessageDeserializeError, MessageOps};
use crate::value_deserializer::Deserialize;
use bytes::BytesMut;
use std::fmt::Debug;

pub fn assert_serialize_eq<T, B>(msg: &T, expected: B)
where
    T: MessageOps + Clone + Debug,
    B: AsRef<[u8]>,
{
    let serialized = msg.clone().serialize_message();
    assert!(serialized.is_ok(), "{msg:#?} didn't serialize");

    let serialized = serialized.unwrap();
    assert_eq!(
        serialized[..],
        *expected.as_ref(),
        "{msg:#?} didn't serialize correctly",
    );
}

pub fn assert_deserialize_eq<T, B>(expected: &T, serialized: B) -> T
where
    T: MessageOps + PartialEq + Debug,
    B: AsRef<[u8]>,
{
    let buf = BytesMut::from(serialized.as_ref());
    let deserialized = T::deserialize_message(buf).unwrap();
    assert_eq!(deserialized, *expected);

    deserialized
}

pub fn assert_deserialize_eq_with_value<T, B, V>(expected: &T, serialized: B, value: &V)
where
    T: MessageOps + PartialEq + Debug,
    B: AsRef<[u8]>,
    V: Deserialize + PartialEq + Debug,
{
    let deserialized = assert_deserialize_eq(expected, serialized);

    assert!(deserialized.kind().has_value());
    let serialized_value = deserialized.value().unwrap();

    let deserialized_value: V = serialized_value.deserialize().unwrap();
    assert_eq!(deserialized_value, *value);
}

#[test]
fn value_larger_than_buffer() {
    let buf = BytesMut::from(&[11, 0, 0, 0, 0, 0, 105, 0, 0, 0, 5][..]);
    assert_eq!(
        Message::deserialize_message(buf),
        Err(MessageDeserializeError::UnexpectedEoi)
    );
}
