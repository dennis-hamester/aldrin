use super::{Message, MessageDeserializeError, MessageOps};
use crate::tags::Tag;
use crate::Deserialize;
use bytes::BytesMut;
use std::fmt::Debug;

#[track_caller]
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

#[track_caller]
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

#[track_caller]
pub fn assert_deserialize_eq_with_value<T, B, V, W>(expected: &T, serialized: B, value: &W)
where
    T: MessageOps + PartialEq + Debug,
    B: AsRef<[u8]>,
    V: Tag,
    W: Deserialize<V> + PartialEq + Debug,
{
    let mut deserialized = assert_deserialize_eq(expected, serialized);

    assert!(deserialized.kind().has_value());

    let serialized_value = deserialized.value().unwrap();
    let deserialized_value: W = serialized_value.deserialize_as().unwrap();
    assert_eq!(deserialized_value, *value);

    let serialized_value = deserialized.value_mut().unwrap();
    let deserialized_value: W = serialized_value.deserialize_as().unwrap();
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
