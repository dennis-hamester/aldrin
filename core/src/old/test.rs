use crate::serialized_value::SerializedValue;
use crate::value_deserializer::Deserialize;
use crate::value_serializer::AsSerializeArg;
use std::fmt::Debug;

/// Asserts that the [`AsSerializeArg`] and [`Deserialize`] impls on `T` match.
///
/// Returns the deserialized value.
///
/// # Panics
///
/// Panics if `T` cannot be deserialized from a serialization of
/// [`<T as AsSerializeArg>::SerializeArg>`](AsSerializeArg::SerializeArg).
#[track_caller]
pub fn assert_as_serialize_arg<T>(value: &T) -> T
where
    T: AsSerializeArg + Deserialize,
{
    assert_as_serialize_arg_with(value)
}

/// Asserts that the [`AsSerializeArg`] impl on `T` and [`Deserialize`] impl on `U` match.
///
/// Returns the deserialized value.
///
/// # Panics
///
/// Panics if `U` cannot be deserialized from a serialization of
/// [`<T as AsSerializeArg>::SerializeArg>`](AsSerializeArg::SerializeArg).
#[track_caller]
pub fn assert_as_serialize_arg_with<T, U>(value: &T) -> U
where
    T: AsSerializeArg + ?Sized,
    U: Deserialize,
{
    let arg = value.as_serialize_arg();

    let serialized = SerializedValue::serialize(&arg)
        .expect("<T as AsSerializeArg>::SerializeArg serializes successfully");

    serialized
        .deserialize::<U>()
        .expect("<T as AsSerializeArg>::SerializeArg deserializes successfully as U")
}

/// Asserts that the [`AsSerializeArg`] and [`Deserialize`] impls on `T` match and that the
/// deserialized value is equal to `value`.
///
/// Returns the deserialized value.
///
/// # Panics
///
/// Panics if either:
/// - `T` cannot be deserialized from a serialization of
///   [`<T as AsSerializeArg>::SerializeArg>`](AsSerializeArg::SerializeArg).
/// - The deserialized `T` does not compare equal to `value`.
#[track_caller]
pub fn assert_as_serialize_arg_eq<T>(value: &T) -> T
where
    T: AsSerializeArg + Deserialize + PartialEq + Debug,
{
    let deserialized = assert_as_serialize_arg(value);
    assert_eq!(*value, deserialized);
    deserialized
}

/// Asserts that the [`AsSerializeArg`] impl on `T` and [`Deserialize`] impl on `U` match and that
/// the deserialized value is equal to `value`.
///
/// Returns the deserialized value.
///
/// # Panics
///
/// Panics if either:
/// - `U` cannot be deserialized from a serialization of
///   [`<T as AsSerializeArg>::SerializeArg>`](AsSerializeArg::SerializeArg).
/// - The deserialized `U` does not compare equal to `value`.
#[track_caller]
pub fn assert_as_serialize_arg_eq_with<T, U>(value: &T) -> U
where
    T: AsSerializeArg + PartialEq<U> + Debug + ?Sized,
    U: Deserialize + Debug,
{
    let deserialized = assert_as_serialize_arg_with(value);
    assert_eq!(*value, deserialized);
    deserialized
}
