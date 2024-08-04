# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Support protocol version 1.18.

## [0.7.0] - 2024-07-25

### Added

- Support protocol version 1.17.
- Support introspection.
- Add `serde` Cargo feature, which enables Serde support in `aldrin-core`.

### Changed

- `Statistics` no longer contains any `pub` fields. Instead, getters have been added for the
  individual statistics. The `non_exhaustive` attribute has also been removed.
- `Statistics` no longer implements `Eq` and `PartialEq`.
- `Statistics` has overall been reduced to contain far fewer fields. It now only counts the number
  of objects, services, channels and bus listeners. Additionally, the number of sent and received
  messages are tracked.

## [0.6.0] - 2024-06-07

- Bump for Aldrin 0.6.0 release.

## [0.5.0] - 2024-05-29

### Added

- Support protocol version 1.16.
- Support aborting calls from the caller's side.

## [0.4.0] - 2024-03-21

### Added

- Support protocol version 15.

### Changed

- Remove the version number in the `EstablishError::IncompatibleVersion` error.
- `BrokerHandle` and `PendingConnection` are adapted to support the new protocol handshake.

### Fixed

- Shut down a connection immediately when an invalid message is received.

## [0.3.0] - 2024-01-18

### Changed

- The return type of `PendingConnection::client_data()` is changed from `&SerializedValue` to
  `&SerializedValueSlice`.
- Rename `ConnectionError::UnexpectedBrokerShutdown` to `UnexpectedShutdown`.
- Rename `EstablishError::VersionMismatch` to `IncompatibleVersion`.
- Rename `EstablishError::BrokerShutdown` to `Shutdown`.

## [0.2.0] - 2023-11-27

### Changed

- Bump for Aldrin 0.2.0.

## [0.1.0] - 2023-11-24

- Initial release.

[0.7.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-broker-0.7.0
[0.6.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-broker-0.6.0
[0.5.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-broker-0.5.0
[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-broker-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-broker-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-broker-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-broker-0.1.0
