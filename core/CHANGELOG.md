# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Add type `UnknownVariant` to represent an unknown enum variant. `ValueSerializer` and
  `ValueDeserializer` have been extended as well.
- Add type `UnknownFields` to represent the unknown fields of a struct. `ValueSerializer` and
  `ValueDeserializer` have been extended as well.
- Enum introspection (`introspection::Enum`) can now specify a special variant as a fallback.
- `ValueSerializer` and `ValueDeserializer` have been extended to support structs with a fallback
  field of type `UnknownFields`.
- Struct introspection (`introspection::Struct`) can now specify a special field as a fallback.
- Service introspection (`introspection::Service`) can now specify a special function as a fallback.
- Service introspection (`introspection::Service`) can now specify a special event as a fallback.

### Changed

- Introspection is now deserialized more liberally and allows struct fields to be out of
  order. Serialization is not changed.
- The closures passed to `StructDeserializer::finish_with()` and `skip_and_finish_with()` now take a
  parameter of type `UnknownFields`.

## [0.10.0] - 2024-11-26

### Added

- Add introspection support for array types.

### Changed

- `BuiltInType` was extended by a new variant `Array`.
- Introspection for arrays for changed from `BuiltInType::Vec` to `BuiltInType::Array`.

## [0.9.0] - 2024-11-19

### Added

- Implement `Serialize` and `Deserialize` for tuples up to size 12.
- `SerializeKey`, `DeserializeKey` and `KeyTypeOf` are now implemented for `ObjectUuid`,
  `ObjectCookie`, `ServiceUuid`, `ServiceCookie`, `ChannelCookie`, `TypeId` and `LexicalId`.
- Add the trait `AsSerializeArg` and helper type alias `SerializeArg`. The trait establishes a
  mapping from serializable types such as `String` to types which are more convenient to pass as
  arguments for serialization such as `&str`. It's expected (but not enforced) that all types, which
  implement `Serialize`, also implement `AsSerializeArg`.

### Changed

- Bump MSRV to 1.71.1.
- The introspection system introduced in 0.7.0 was completely redone and is not backwards
  compatible. Generally, all types that are `Serialize` now also implement `Introspectable`.
- The `SerializeKey` and `DeserializeKey` traits have been redesigned and it's now possible to
  implement them for custom types. The new traits resemble specialized versions of `Into` and
  `TryFrom` and map custom types to and from one of the base key types.
- The `is_nil()` method of all id types now takes `self` by value instead of by reference.

## [0.8.0] - 2024-09-22

### Added

- Add new protocol version 1.18.
- Add `SubscribeService`, `SubscribeServiceReply` and `UnsubscribeService` messages.
- Add `subscribe_all` field to `ServiceInfo`.
- Add `SubscribeAllEvents`, `SubscribeAllEventsReply`, `UnsubscribeAllEvents` and
  `UnsubscribeAllEventsReply` messages.
- Implement `Debug` for `BoxedTransport<'a, E>`.

### Changed

- All fields of `ServiceInfo` are now private to better support future extensions.

## [0.7.0] - 2024-07-25

### Added

- `AsyncTransportExt::boxed()` turns an `AsyncTransport` into a
  `Pin<Box<dyn AsyncTransport + Send>>`. A type alias `BoxedTransport` has also been added.
- Add an `impl<'a, T: AsRef<[u8]>> From<&'a T>` for `&'a ByteSlice`. This removes the need to import
  `ByteSlice` is most cases.
- Add optional Serde support for `Value` and all id types. It is gated behind the `serde` Cargo
  feature.
- Add optional introspection support, which is gated behind the `introspection` Cargo feature. If
  enabled, this adds the `introspection` module containing all relevant types exchanged by brokers
  and clients.

### Changed

- Bump protocol version to 1.17.
- `AsyncTransportExt` now takes `impl Into<Message>` in `send()`, `send_and_flush()` and
  `send_start_unpin()`.
- Enable the `std` feature of `uuid`.

## [0.6.0] - 2024-06-07

- Bump for Aldrin 0.6.0 release.

## [0.5.0] - 2024-05-29

### Added

- Add `DeserializeError::MoreElementsRemain`.
- Add `FieldDeserializer::try_id()`.
- Add `EnumDeserializer::try_variant()`.
- Add `StructDeserializer::deserialize_specific_field()`.
- Add `VecDeserializer::finish()`, `finish_with()`, `skip_and_finish()` and
  `skip_and_finish_with()`.
- Add `BytesDeserializer::finish()`, `finish_with()`, `skip_and_finish()` and
  `skip_and_finish_with()`.
- Add `MapDeserializer::finish()`, `finish_with()`, `skip_and_finish()` and
  `skip_and_finish_with()`.
- Add `SetDeserializer::finish()`, `finish_with()`, `skip_and_finish()` and
  `skip_and_finish_with()`.
- Add `StructDeserializer::finish()`, `finish_with()`, `skip_and_finish()` and
  `skip_and_finish_with()`.

### Changed

- Bump protocol version to 16.
- `StructSerializer::serialize_field()` takes `id` now as `impl Into<u32>`.
- `Serializer::serialize_enum()` takes `variant` now as `impl Into<u32>`.
- Replace `VecDeserializer::remaining_elements()` and `has_more_elements()` with `len()` and
  `is_empty()`.
- Replace `MapDeserializer::remaining_elements()` and `has_more_elements()` with `len()` and
  `is_empty()`.
- Replace `SetDeserializer::remaining_elements()` and `has_more_elements()` with `len()` and
  `is_empty()`.

## [0.4.0] - 2024-03-21

### Added

- Implement `Serialize` and `Deserialize` for `Result`.
- Implement `Serialize` and `Deserialize` for `std::convert::Infallible`.
- Add `NIL` and `is_nil()` to all id types.
- Implement `Default` for all id types.
- Add type `ProtocolVersion`.
- Define new connection messages `Connect2` and `ConnectReply2`, that will allow future protocol
  extensions to be backward-compatible.

### Changed

- Bump protocol version to 15.
- Rename `ConnectReply::VersionMismatch` to `IncompatibleVersion`.
- Remove `VERSION` constant. Use the associated constants `MIN` and `MAX` of `ProtocolVersion`
  instead.

## [0.3.0] - 2024-01-18

- Bump for Aldrin 0.3.0 release.

## [0.2.0] - 2023-11-27

### Added

- Implement `FromStr` for `ObjectUuid` and `ServiceUuid`.

## [0.1.0] - 2023-11-24

- Initial release.

[0.10.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.10.0
[0.9.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.9.0
[0.8.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.8.0
[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.1.0
