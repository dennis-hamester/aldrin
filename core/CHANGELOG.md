# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Add new protocol version 1.18.
- Add `SubscribeService` and `SubscribeServiceReply` messages.

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

[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.1.0
