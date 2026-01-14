# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Fixed

- Backport fixes for a deadlock from 0.13.0.

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

[0.4.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-broker-0.4.0
[0.3.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-broker-0.3.0
[0.2.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-broker-0.2.0
[0.1.0]: https://github.com/dennis-hamester/aldrin/releases/tag/aldrin-broker-0.1.0
