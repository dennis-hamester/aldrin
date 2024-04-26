# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Add `FieldDeserializer::try_id()`.
- Add `EnumDeserializer::try_variant()`.
- Add `StructDeserializer::deserialize_specific_field()`.

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

[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-core-0.1.0
